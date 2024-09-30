#[cfg(test)]
mod integration_final {
    mod example {
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

        pub fn start() {
            let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to start server.");

            let router = Router::new()
                .route("/", get(hello_world))
                .route("/post/:id", post(post_handler))
                .route("/cpu", get(cpu_bound_task));
            cairo::serve(listener, router);
        }
    }

    use std::{
        io::{self, Read, Write},
        net::TcpStream,
        thread::{sleep, spawn},
        time::Duration,
    };

    const HOST_AND_PORT: &'static str = "localhost:7878";

    fn curl(method_and_path: &str) -> io::Result<String> {
        let mut stream = TcpStream::connect(HOST_AND_PORT)?;

        let request = format!(
            "{} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            method_and_path
        );
        stream.write_all(request.as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        Ok(response)
    }

    /// Send requests using a raw `TcpStream` to confirm our server is correct at the lower level.
    fn assert_tcp_stream() {
        let response = curl("GET /").expect("Failed to make HTTP call.");
        assert_eq!(
            response,
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!"
        );

        let response = curl("GET /cpu").expect("Failed to make HTTP call.");
        assert_eq!(
            response,
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nTotal: 1000000"
        );

        let response = curl("GET /goodbye").expect("Failed to make HTTP call.");
        assert_eq!(
            response,
            "HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/plain\r\n\r\nNot Found"
        );
    }

    /// Send requests using `ureq` to confirm our server works with third-party HTTP libraries.
    fn assert_ureq() {
        let url = format!("http://{}", HOST_AND_PORT);

        let response = ureq::get(&format!("{}/", url)).call().unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers_names(), vec!["content-type".to_string()]);
        assert_eq!(response.header("content-type"), Some("text/plain"));
        assert_eq!(response.into_string().unwrap(), "Hello, World!".to_string());

        let response = ureq::post(&format!("{}/post/44", url))
            .send_bytes(b"Hello Rust")
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers_names(), vec!["content-type".to_string()]);
        assert_eq!(response.header("content-type"), Some("text/plain"));
        assert_eq!(
            response.into_string().unwrap(),
            "ID: 44, Body: Hello Rust".to_string()
        );

        let ureq::Error::Status(400, _response) =
            ureq::post(&format!("{}/post/44", url)).call().unwrap_err()
        else {
            panic!("Expected a 400.");
        };

        let response = ureq::get(&format!("{}/cpu", url)).call().unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers_names(), vec!["content-type".to_string()]);
        assert_eq!(response.header("content-type"), Some("text/plain"));
        assert_eq!(
            response.into_string().unwrap(),
            "Total: 1000000".to_string()
        );

        let ureq::Error::Status(404, response) =
            ureq::get(&format!("{}/goodbye", url)).call().unwrap_err()
        else {
            panic!("Expected a 404.");
        };
        assert_eq!(response.status(), 404);
        assert_eq!(response.headers_names(), vec!["content-type".to_string()]);
        assert_eq!(response.header("content-type"), Some("text/plain"));
        assert_eq!(response.into_string().unwrap(), "Not Found".to_string());
    }

    #[test]
    fn test_the_server_works() {
        // We only start a single instance of the server to avoid any port conflicts.
        spawn(|| example::start());
        sleep(Duration::from_millis(100));

        assert_tcp_stream();
        assert_ureq();
    }
}
