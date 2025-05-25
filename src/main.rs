use shirry_webserver::ThreadPool;
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

// We could do more here! If you want to continue enhancing this project, here are some ideas:
//     1. Add more documentation to ThreadPool and its public methods.
//     2. Add tests of the library’s functionality.
//     3. Change calls to unwrap to more robust error handling.
//     4. Use ThreadPool to perform some task other than serving web requests.
//     5. Find a thread pool crate on crates.io and implement a similar web server using the crate
//        instead. Then compare its API and robustness to the thread pool we implemented.

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(3);

    // Shut down after processing 10 requests to test exit logic
    for stream in listener.incoming().take(10) {
        let stream = stream.unwrap();

        pool.execute(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
