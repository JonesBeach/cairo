use std::net::TcpListener;

use cairo::{
    extract::Path,
    routing::{get, post},
    Router,
};

fn hello_world() -> &'static str {
    "Hello, World!"
}

fn post_handler(Path(id): Path<usize>, body: String) -> String {
    format!("ID: {}, Body: {}", id, body)
}

fn cpu_bound_task() -> String {
    let mut total = 0;
    for _ in 0..1_000_000 {
        total += 1;
    }
    format!("Total: {}", total)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to start server.");
    println!("Server listening on port 7878");

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/post/:id", post(post_handler))
        .route("/cpu", get(cpu_bound_task));
    cairo::serve(listener, router);
}
