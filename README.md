# Motivations
The goal behind this project was to get some practice with the foundations of concurrency in Rust. The project's core idea is implementing a web server that is able to serve multiple requests at once.

# Thread Pool Approach
This approach relies on multithreading principles to serve requests. A thread pool containing workers act as the engine of the server. Jobs are received per request and passed to whichever worker is available.

# Async Approach
This approach aimed to deliver all the functional features of the multithreading approach, but leveraging async runtimes. Some basic practice with the tokio crate and state handling of async tasks through JoinHandle manipulations.
