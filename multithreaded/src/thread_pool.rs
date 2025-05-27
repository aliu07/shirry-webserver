use super::{Job, worker::Worker};
use std::sync::{Arc, Mutex, mpsc};

pub struct ThreadPool {
    workers: Vec<Worker>,
    // () is the return type of the closure we pass to each worker thread
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

    /// Executes a given job. Wraps the job behind a Box pointer and passes
    /// the pointer into the channel. On the receiving end, one of the worker
    /// threads in the thread pool will pick up the pointer, unwrap it, and
    /// execute the job closure.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Custom `Drop` implementation for `ThreadPool`.
    ///
    /// This implementation ensures that the `ThreadPool` is properly shut down
    /// when it goes out of scope. It performs the following cleanup actions:
    ///
    /// 1. Closes the sender side of the channel, which signals the worker threads
    ///    to stop processing tasks. When the sender is dropped, the channel is closed,
    ///    causing the receivers in the worker threads to throw errors and halt execution.
    ///
    /// 2. Transfers ownership of the thread `JoinHandle` from each worker to the main thread
    ///    using `drain()`. This allows the main thread to call `join()` on each worker thread,
    ///    ensuring that all threads complete their execution before the `ThreadPool` is dropped.
    ///
    /// # Behavior
    /// - Each worker thread is shut down gracefully by joining its thread handle.
    /// - A message is printed to indicate that a worker is being shut down, along with its ID.
    ///
    /// # Constraints
    /// - This implementation assumes that all worker threads are properly initialized and
    ///   have valid `JoinHandle`s.
    /// - The `sender` field must be wrapped in an `Option` to allow ownership transfer via `take()`.
    ///
    /// # Potential Issues
    /// - If a worker thread panics during execution, calling `join()` will propagate the panic
    ///   to the main thread. This implementation uses `unwrap()` on the result of `join()`,
    ///   which will cause the program to terminate if a worker thread panicked.
    /// - Dropping the `ThreadPool` while tasks are still being processed may result in incomplete
    ///   task execution, as the workers will halt when the channel is closed.
    ///
    /// # Example
    /// ```rust
    /// {
    ///     let pool = ThreadPool::new(4);
    ///     // Use the thread pool for tasks...
    /// } // ThreadPool is dropped here, and all worker threads are shut down.
    /// ```
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
