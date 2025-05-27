use super::Job;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new `Worker` instance with the specified ID and a shared receiver for jobs.
    ///
    /// This method spawns a new thread that continuously listens for incoming jobs from the
    /// provided `receiver`. Each job is executed by the worker thread. If the `receiver` is
    /// disconnected (e.g., the sender is dropped), the worker thread will shut down gracefully.
    ///
    /// # Arguments
    /// - `id`: A unique identifier for the worker. This ID is used for logging and debugging purposes.
    /// - `receiver`: An `Arc<Mutex<mpsc::Receiver<Job>>>` that allows the worker thread to receive jobs
    ///   from a shared channel. The `Arc` ensures shared ownership, and the `Mutex` ensures thread-safe
    ///   access to the receiver.
    ///
    /// # Returns
    /// A new `Worker` instance with the specified ID and an active thread for processing jobs.
    ///
    /// # Behavior
    /// - The worker thread runs in an infinite loop, waiting for jobs from the `receiver`.
    /// - When a job is received, the worker logs its ID and executes the job.
    /// - If the `receiver` is disconnected, the worker logs its ID and shuts down gracefully.
    ///
    /// # Example
    /// ```rust
    /// use std::{
    ///     sync::{Arc, Mutex, mpsc},
    /// };
    ///
    /// let (_, rx) = mpsc::channel();
    /// let receiver = Arc::new(Mutex::new(rx));
    /// let worker = Worker::new(1, Arc::clone(&receiver));
    ///
    /// assert_eq!(worker.get_id(), 1);
    /// ```
    ///
    /// # Notes
    /// - The `thread` field of the `Worker` is wrapped in an `Option` to allow ownership transfer
    ///   via the `take_thread()` method.
    /// - The worker thread will block while waiting for jobs, so ensure that jobs are sent to the
    ///   channel to keep the worker active.
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }

    /// Returns the unique identifier of the `Worker`.
    ///
    /// This method provides access to the `id` field of the `Worker`, which is used for
    /// logging, debugging, and distinguishing between different workers in a thread pool.
    ///
    /// # Returns
    /// A `usize` representing the unique ID of the worker.
    ///
    /// # Example
    /// ```rust
    /// use std::{
    ///     sync::{Arc, Mutex, mpsc},
    /// };
    ///
    /// let (_, rx) = mpsc::channel();
    /// let receiver = Arc::new(Mutex::new(rx));
    /// let worker = Worker::new(1, Arc::clone(&receiver));
    ///
    /// assert_eq!(worker.get_id(), 1);
    /// ```
    ///
    /// # Notes
    /// - The `id` is assigned when the `Worker` is created using the `new()` method.
    /// - This method does not modify the state of the `Worker`.
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Transfers ownership of the worker's thread handle.
    ///
    /// This method consumes the `thread` field of the `Worker` and returns the `JoinHandle`
    /// associated with the worker's thread. After calling this method, the `Worker` no longer
    /// holds the thread handle, and the `thread` field is set to `None`.
    ///
    /// # Returns
    /// A `thread::JoinHandle<()>` representing the worker's thread handle.
    ///
    /// # Panics
    /// This method will panic if the `thread` field is `None`. Ensure that the `Worker` still
    /// holds a valid thread handle before calling this method.
    ///
    /// # Example
    /// ```rust
    /// use std::{
    ///     sync::{Arc, Mutex, mpsc},
    /// };
    ///
    /// let (_, rx) = mpsc::channel();
    /// let receiver = Arc::new(Mutex::new(rx));
    /// let mut worker = Worker::new(1, Arc::clone(&receiver));
    ///
    /// let handle = worker.take_thread();
    /// assert!(worker.thread.is_none());
    ///
    /// // Wait for the worker thread to finish
    /// handle.join().unwrap();
    /// ```
    ///
    /// # Notes
    /// - This method is typically used to shut down the worker thread gracefully by calling
    ///   `join()` on the returned `JoinHandle`.
    /// - The `thread` field is wrapped in an `Option` to allow ownership transfer via `take()`.
    pub fn take_thread(&mut self) -> thread::JoinHandle<()> {
        self.thread.take().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_worker_id() {
        let (_, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let worker = Worker::new(5, Arc::clone(&receiver));

        assert_eq!(worker.get_id(), 5);
    }

    #[test]
    fn take_thread_from_worker() {
        let (_, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut worker = Worker::new(5, Arc::clone(&receiver));

        worker.take_thread();

        assert!(worker.thread.is_none());
    }

    #[test]
    fn worker_executes_job() {
        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        let mut worker = Worker::new(5, Arc::clone(&receiver));

        let result = Arc::new(Mutex::new(0));

        {
            let result = Arc::clone(&result);
            let job = Box::new(move || {
                let mut res = result.lock().unwrap();
                *res += 5;
            });

            tx.send(job).unwrap();
        }

        // Disconnect worker
        drop(tx);

        // Wait for worker to finish job
        worker.take_thread().join().unwrap();

        let res = result.lock().unwrap();
        assert_eq!(*res, 5);
    }
}
