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

    pub fn get_id(&self) -> usize {
        self.id
    }

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
