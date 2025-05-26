pub use self::thread_pool::ThreadPool;

mod thread_pool;
mod worker;

// Custom type alias
type Job = Box<dyn FnOnce() + Send + 'static>;
