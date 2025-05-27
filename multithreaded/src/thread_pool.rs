use super::{Job, worker::Worker};
use std::sync::{Arc, Mutex, mpsc};

pub struct ThreadPool {
    // () is the return type of the closure we pass to each worker thread
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        // Pre-allocates space for the vector... faster than using new() and dynamically
        // sizing the vector with push
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // When we drop the sender, channel will close. Receivers in worker threads
        // will throw errors causing them to come to a halt.
        drop(self.sender.take());

        // We need to pass ownership of the thread JoinHandle outside of
        // each worker so that join() can consume them. We transfer ownership
        // by calling drain().
        for mut worker in &mut self.workers.drain(..) {
            println!("Shutting down worker {}", worker.get_id());

            worker.take_thread().join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn thread_pool_panics_on_zero_size() {
        let _pool = ThreadPool::new(0);
    }

    #[test]
    fn init_valid_thread_pool() {
        let pool = ThreadPool::new(3);

        assert_eq!(pool.workers.len(), 3);
    }

    #[test]
    fn thread_pool_executes_jobs_and_exits() {
        let pool = ThreadPool::new(3);
        let result = Arc::new(Mutex::new(0));

        for _ in 0..8 {
            let result = Arc::clone(&result);
            pool.execute(move || {
                let mut res = result.lock().unwrap();
                *res += 1;
            });
        }

        // Call drop to join all the threads before asserting final result
        drop(pool);

        let res = *result.lock().unwrap();
        assert_eq!(res, 8);
    }
}
