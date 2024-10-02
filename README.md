# cairo
**Cairo** is the final module in a Rust HTTP server course from [From Scratch Code](https://fromscratchcode.com/), designed as a hands-on learning tool for intermediate Rust developers. This project allows you to explore core Rust concepts by building a simple, Axum-inspired web framework and HTTP server from scratch, with no third-party libraries. Whether you're looking to deepen your understanding of network programming or just want a practical project to enhance your Rust-y skills, Cairo hopes to always be there for you.

## Features
This project includes:
- A synchronous TCP/HTTP server in Rust
- The HTTP spec
- Compiling your crate as a library
- Declarative Rust macros using `macro_rules`
- Extractors for request data
- Dynamic HTTP routing using type erasure
- Threads and a thread pool

## Example Usage
Here's a simple example of setting up routes and handlers using Cairo. Running this example will start a local server, returning a greeting on the root path and handling `POST` requests to `/post/:id`.
```rust
// examples/server_final.rs
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

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to start server.");

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/post/:id", post(post_handler))
    cairo::serve(listener, router);
}
```

## Local Development
To test or run the server locally:
```bash
# Run the tests
cargo test

# Run the example server
cargo run --example server_final
```

## Course Information
Interested in building this project step-by-step? [Check out the full course](https://fromscratchcode.com/courses/), where we build the HTTP server from scratch, covering everything from basic routing to advanced topics like Rust macros and concurrency.

The course is perfect for Rust developers who want to deepen their skills in server-side programming, learn about HTTP servers and routing without relying on third-party libraries, and build advanced library interfaces.

## Disclaimer
**Important Notice**: This project is currently in active development and is considered experimental. It may lack certain production-level features such as robust error handling or security protections. Use as a learning or testing tool **only** is encouraged.

## License
The software is granted under the terms of the MIT License. For details see the file `LICENSE` included with the source distribution. All copyrights are owned by their respective authors.

## Attribution
This project uses software components that are licensed under the MIT License. While we do not use Axum directly, several components (particularly the `Router`, `PathRouter`, `Handlder`, `FromRequest`, and `FromRequestParts`) are heavily inspired by Axum. The following is the required attribution notice for the original work:

```
MIT License

Copyright (c) 2024 Tokio Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

This notice applies to the following components used in this project:

- Axum by Tokio Contributors (https://github.com/tokio-rs/axum)

By including this notice, we comply with the terms of the MIT License and ensure that proper credit is given to the original authors of the software components used in this project.
