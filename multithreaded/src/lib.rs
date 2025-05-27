pub use self::thread_pool::ThreadPool;
pub use self::worker::Worker;

mod thread_pool;
mod worker;

// Custom type alias
type Job = Box<dyn FnOnce() + Send + 'static>;
