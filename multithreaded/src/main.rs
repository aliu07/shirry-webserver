use multithreaded::ThreadPool;
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    process, thread,
    time::Duration,
};

static ADDRESS: &str = "127.0.0.1";
static DEFAULT_PORT: &str = "7878";

fn main() {
    let listener = bind_listener(ADDRESS, DEFAULT_PORT);
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

    let request_line = match buf_reader.lines().next().unwrap_or_else(|| {
        eprintln!("[ERROR] No lines found in buffer");
        process::exit(1);
    }) {
        Ok(line) => line,
        Err(err) => {
            eprintln!("[ERROR] Failed to read request line: {err}");
            return;
        }
    };

    let response = generate_response(&request_line);

    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("[ERROR] Failed to write response: {err}");
    };
}

fn bind_listener(address: &str, port: &str) -> TcpListener {
    match TcpListener::bind(format!["{}:{}", address, port]) {
        Ok(binding) => {
            println!("[INFO] Succesfully bound listener to port {DEFAULT_PORT}");
            binding
        }
        Err(err) => {
            eprintln!("[ERROR] Failed to bind listener on port {DEFAULT_PORT}: {err}");

            // Let OS handle which port to bind listener to
            eprintln!("Retrying to bind; letting OS assign port");
            let retry = TcpListener::bind(format!["{}:0", ADDRESS]);

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
    }
}

fn generate_response(request_line: &str) -> String {
    let (status_line, file_path) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "../pages/index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "../pages/sleep.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "../pages/404.html"),
    };

    let contents = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("[ERROR] Failed to read file: {err}");
        process::exit(1);
    });
    let length = contents.len();

    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}")
}
