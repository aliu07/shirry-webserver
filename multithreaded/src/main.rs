use multithreaded::ThreadPool;
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    process, thread,
    time::Duration,
};

// We could do more here! If you want to continue enhancing this project, here are some ideas:
//     1. Add more documentation to ThreadPool and its public methods.
//     2. Change calls to unwrap to more robust error handling.
//     3. Use ThreadPool to perform some task other than serving web requests.
//     4. Find a thread pool crate on crates.io and implement a similar web server using the crate
//        instead. Then compare its API and robustness to the thread pool we implemented.

fn main() {
    let address = "127.0.0.1";
    let port = "7878";
    let result = TcpListener::bind(format!["{}:{}", address, port]);

    let listener = match result {
        Ok(binding) => {
            println!("[INFO] Succesfully bound listener to port {port}");
            binding
        }
        Err(err) => {
            eprintln!("[ERROR] Failed to bind listener on port {port}: {err}");

            // Let OS handle which port to bind listener to
            eprintln!("Retrying to bind; letting OS assign port");
            let retry = TcpListener::bind(format!["{}:0", address]);

            match retry {
                Ok(binding) => {
                    let new_port = binding.local_addr().unwrap().port();
                    println!("Succesfully bound listener to port {new_port}");
                    binding
                }
                Err(err) => {
                    eprintln!("[ERROR] Failed to bind listener again: {err}; exiting");
                    process::exit(1);
                }
            }
        }
    };

    let pool = ThreadPool::new(3);

    // Shut down after processing 10 requests to test exit logic
    for stream in listener.incoming().take(10) {
        let stream = stream.unwrap_or_else(|err| {
            eprintln!("[ERROR] Failed to fetch next item in stream: {err}");
            process::exit(1);
        });

        pool.execute(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request_line = buf_reader.lines().next();
    // Shadow previous binding
    let request_line = match request_line {
        Some(Ok(line)) => line,
        Some(Err(err)) => {
            eprintln!("[ERROR] Failed to read request line: {err}");
            process::exit(1);
        }
        None => {
            eprintln!("[ERROR] No lines found in buffer");
            process::exit(1);
        }
    };

    let (status_line, file_path) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "../pages/index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "../pages/index.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "../pages/404.html"),
    };

    let contents = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("[ERROR] Failed to read file: {err}");
        process::exit(1);
    });
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("[ERROR] Failed to write response: {err}");
        process::exit(1);
    };
}
