use axum::{Router, response::Html, routing::get};
use std::{process, time::Duration};
use tokio::{fs, net::TcpListener, time};

static ADDRESS: &str = "127.0.0.1";
static DEFAULT_PORT: &str = "7878";

#[tokio::main]
async fn main() {
    let app = Router::new()
        // `GET /` goes to the `root` handler
        .route("/", get(root))
        // `GET /sleep` goes to the `sleep` handler
        .route("/sleep", get(sleep))
        // Anything else goes to the `not_found` handler
        .fallback(not_found);

    let listener = bind_listener(ADDRESS, DEFAULT_PORT).await;

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<String> {
    Html(parse_file("../pages/index.html").await)
}

async fn sleep() -> Html<String> {
    time::sleep(Duration::from_secs(10)).await;
    Html(parse_file("../pages/sleep.html").await)
}

async fn not_found() -> Html<String> {
    Html(parse_file("../pages/404.html").await)
}

async fn parse_file(file_path: &str) -> String {
    let contents = fs::read_to_string(file_path).await.unwrap_or_else(|err| {
        eprintln!("[ERROR] Failed to read file: {err}");
        process::exit(1);
    });

    contents
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
