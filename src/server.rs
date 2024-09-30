use std::{
    io::{self, ErrorKind, Read, Write},
    net::TcpListener,
    str,
    sync::Arc,
};

use crate::{
    core::ThreadPool,
    http::{Request, Response},
    Router,
};

/// An HTTP request may require multiple reads from a stream. Here we read from a stream until we
/// have read the entirety of the HTTP headers and body and return the resulting buffer.
fn fill_buffer<T: Read>(stream: &mut T) -> io::Result<Vec<u8>> {
    let mut buffer = vec![];
    let mut temp_buffer = [0; 512];
    let mut headers_complete = false;
    let mut content_length = 0;

    while !headers_complete {
        let num_bytes_read = stream.read(&mut temp_buffer)?;
        println!("Read {} bytes.\n--", num_bytes_read);
        if num_bytes_read == 0 {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "Zero bytes read."));
        }
        buffer.extend_from_slice(&temp_buffer[..num_bytes_read]);

        // Check if we've read the headers completely
        if let Some(headers_end_pos) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            headers_complete = true;
            let headers_str = str::from_utf8(&buffer[..headers_end_pos])
                .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

            // Parse headers to get the Content-Length. If we do not find one, we use the default
            // defined above of 0.
            for line in headers_str.split("\r\n") {
                if line.starts_with("Content-Length:") {
                    let length_str = line
                        .split(':')
                        .nth(1)
                        .ok_or_else(|| {
                            io::Error::new(ErrorKind::InvalidData, "Invalid Content-Length header")
                        })?
                        .trim();
                    content_length = length_str
                        .parse::<usize>()
                        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
                    break;
                }
            }

            // If there is a body, calculate remaining bytes to read, given that "\r\n\r\n" is 4
            // bytes. We only need this clause for when the body is split across streams. When the
            // body is entirely in the next stream read, then `buffer.len() == body_start_pos`.
            let body_start_pos = headers_end_pos + 4;
            if buffer.len() > body_start_pos {
                content_length -= buffer.len() - body_start_pos;
            }
        }
    }

    // If there is a body, read the remaining bytes. We do this in a loop in case it is sent across
    // multiple streams.
    while content_length > 0 {
        let num_bytes_read = stream.read(&mut temp_buffer)?;
        println!("Read {} bytes.\n--", num_bytes_read);
        if num_bytes_read == 0 {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "Zero bytes read."));
        }
        buffer.extend_from_slice(&temp_buffer[..num_bytes_read]);
        content_length -= num_bytes_read;
    }

    Ok(buffer)
}

/// Read from a `TcpStream` (or any type that implements `Read`) and attempt to get an HTTP
/// `Request`.
fn parse_request<T: Read>(stream: &mut T) -> io::Result<Request> {
    let buffer = fill_buffer(stream)?;

    // By this point, we know we have read our headers and body into the `buffer`.
    let request = str::from_utf8(&buffer).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    println!("Request: {}", request);

    Request::try_from(request)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Unexpected request format."))
}

/// Write an HTTP `Response` to a `TcpStream` (or any type that implements `Write`).
fn send_response<T: Write>(stream: &mut T, response: Response) -> io::Result<usize> {
    let num_bytes_written = stream.write(&response.as_bytes())?;
    stream.flush()?;

    Ok(num_bytes_written)
}

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
            Ok(mut stream) => {
                let router = router.clone();
                // We must `move` the `Arc<Router>` into the closure since it could outlive this
                // function.
                pool.execute(move || {
                    let request = match parse_request(&mut stream) {
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

                    match send_response(&mut stream, response) {
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::http::Method;

    #[test]
    fn test_parse_request_valid_root() {
        let mut stream = Cursor::new(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
        let request = parse_request(&mut stream).unwrap();
        assert_eq!(
            request,
            Request::with_headers(
                Method::Get,
                "/",
                vec![("Host".to_string(), "localhost".to_string())]
            )
        );
    }

    #[test]
    fn test_parse_request_invalid_utf8() {
        let mut stream = Cursor::new(b"\x80\x81\x82\x83");
        let result = parse_request(&mut stream);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);

        let mut stream = Cursor::new(b"\x80\x81\x82\x83\r\n\r\n");
        let result = parse_request(&mut stream);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_parse_request_empty() {
        let mut stream = Cursor::new(b"");
        let result = parse_request(&mut stream);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_parse_request_invalid_format() {
        let mut stream = Cursor::new(b"INVALID REQUEST\r\n");
        let result = parse_request(&mut stream);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);

        let mut stream = Cursor::new(b"INVALID REQUEST\r\n\r\n");
        let result = parse_request(&mut stream);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_send_response_ok() {
        let response = Response::new(
            200,
            vec![("Content-Type".to_string(), "text/plain".to_string())],
            "Hello, World!".to_string(),
        );
        let mut cursor = Cursor::new(vec![]);

        let result = send_response(&mut cursor, response);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 58);
        assert_eq!(
            cursor.into_inner(),
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!"
        );
    }

    #[test]
    fn test_send_response_io_error() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, "write failed"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::new(io::ErrorKind::Other, "flush failed"))
            }
        }

        let response = Response::new(200, vec![], "Hello, World!".to_string());
        let mut failing_writer = FailingWriter;

        let result = send_response(&mut failing_writer, response);

        assert!(result.is_err());
    }
}
