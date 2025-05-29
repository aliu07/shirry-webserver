use futures::future;
use std::{error::Error, process, time::Duration};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time,
};

static ADDRESS: &str = "127.0.0.1";
static DEFAULT_PORT: &str = "7878";
static NUM_TASKS: usize = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = bind_listener(ADDRESS, DEFAULT_PORT).await;
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(NUM_TASKS);

    for i in 0..NUM_TASKS {
        let (stream, address) = listener.accept().await.unwrap_or_else(|err| {
            eprintln!("[ERROR] Failed to fetch next item in stream: {err}");
            process::exit(1);
        });

        let handle = tokio::spawn(async move {
            println!("[EVENT] Received request {i} on socket address {address}");

            handle_connection(stream).await;
        });

        handles.push(handle);
    }

    future::join_all(handles).await;

    Ok(())
}

async fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let buffer = buf_reader.lines().next_line().await.unwrap_or_else(|err| {
        eprintln!("[ERROR] Failed to read request line: {err}");
        process::exit(1);
    });

    let request_line = match buffer {
        Some(line) => line,
        None => {
            eprintln!("[ERROR] No lines found in buffer");
            return;
        }
    };

    let response = generate_response(&request_line).await;

    if let Err(err) = stream.write_all(response.as_bytes()).await {
        eprintln!("[ERROR] Failed to write response: {err}");
    };
}

async fn bind_listener(address: &str, port: &str) -> TcpListener {
    match TcpListener::bind(format!["{}:{}", address, port]).await {
        Ok(binding) => {
            println!("[INFO] Succesfully bound listener to port {DEFAULT_PORT}");
            binding
        }
        Err(err) => {
            eprintln!("[ERROR] Failed to bind listener on port {DEFAULT_PORT}: {err}");

            // Let OS handle which port to bind listener to
            println!("[INFO] Retrying to bind; letting OS assign port");

            match TcpListener::bind(format!["{}:0", ADDRESS]).await {
                Ok(binding) => {
                    let new_port = binding.local_addr().unwrap().port();
                    println!("[INFO] Succesfully bound listener to port {new_port}");
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

async fn generate_response(request_line: &str) -> String {
    let (status_line, file_path) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "../pages/index.html"),
        "GET /sleep HTTP/1.1" => {
            time::sleep(Duration::from_secs(10)).await;
            ("HTTP/1.1 200 OK", "../pages/sleep.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "../pages/404.html"),
    };

    let contents = fs::read_to_string(file_path).await.unwrap_or_else(|err| {
        eprintln!("[ERROR] Failed to read file: {err}");
        process::exit(1);
    });
    let length = contents.len();

    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}")
}
