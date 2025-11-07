use std::{io::ErrorKind, net::TcpListener, sync::Arc};

use crate::{connection::Connection, core::ThreadPool, Router};

/// Serve incoming TCP connections using the provided `Router`.
///
/// This function listens for incoming TCP connections on the given `TcpListener` and uses a
/// thread pool to handle each connection concurrently. Each connection is parsed into a
/// `Request`, which is then routed using the `Router`. The resulting `Response` is sent back to
/// the client.
pub fn serve(listener: TcpListener, router: Router) {
    // We create an `Arc` so we can share the `Router` between threads.
    let router = Arc::new(router);
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let router = router.clone();
                // We must `move` the `Arc<Router>` into the closure since it could outlive this
                // function.
                pool.execute(move || {
                    let mut conn = Connection::new(stream);
                    let request = match conn.read_request() {
                        Ok(request) => request,
                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                            // Ignoring UnexpectedEof error, this occurs when we read zero bytes,
                            // which indicates the client has closed a connection.
                            println!("Client closed connection.");
                            return;
                        }
                        Err(e) => {
                            eprintln!("An error occurred: {}", e);
                            return;
                        }
                    };

                    // Turn the HTTP `Request` into the `Response` using the `Router` which will
                    // call the appropriate handler.
                    let response = router.call(request);

                    match conn.send_response(response) {
                        Ok(num_bytes_written) => {
                            println!("--\nSent {} bytes.\n--", num_bytes_written);
                        }
                        Err(e) => {
                            eprintln!("An error occurred: {}", e);
                        }
                    }
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
}
