use std::{
    io::{Error, ErrorKind, Result},
    str,
};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::http::{Request, Response};

pub fn parse_request_from_bytes(bytes: &[u8]) -> Result<Request> {
    let text = str::from_utf8(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    Request::parse(text)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Unexpected request format."))
}

pub(crate) struct Connection<T: AsyncRead + AsyncWrite> {
    stream: T,
}

impl<T: AsyncRead + AsyncWrite + Unpin> Connection<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    /// Read from an async stream and attempt to get an HTTP `Request`.
    pub async fn read_request(&mut self) -> Result<Request> {
        let bytes = self.read_bytes().await?;
        parse_request_from_bytes(&bytes)
    }

    /// Write an HTTP `Response` to an async stream.
    pub async fn send_response(&mut self, response: Response) -> Result<()> {
        self.write(&response.as_bytes()).await
    }

    async fn write(&mut self, bytes: &[u8]) -> Result<()> {
        let _ = self.stream.write(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// An HTTP request may require multiple reads from a stream. Here we read from a stream until
    /// we have read the entirety of the HTTP headers and body and return the resulting buffer.
    async fn read_bytes(&mut self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        let mut temp_buffer = [0; 512];
        let mut headers_complete = false;
        let mut content_length = 0;

        while !headers_complete {
            let num_bytes_read = self.stream.read(&mut temp_buffer).await?;
            if num_bytes_read == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Zero bytes read."));
            }
            buffer.extend_from_slice(&temp_buffer[..num_bytes_read]);

            // Check if we've read the headers completely
            if let Some(headers_end_pos) =
                buffer.windows(4).position(|window| window == b"\r\n\r\n")
            {
                headers_complete = true;
                let headers_str = str::from_utf8(&buffer[..headers_end_pos])
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

                // Parse headers to get the Content-Length. If we do not find one, we use the
                // default defined above of 0.
                for line in headers_str.split("\r\n") {
                    if line.starts_with("Content-Length:") {
                        let length_str = line
                            .split(':')
                            .nth(1)
                            .ok_or_else(|| {
                                Error::new(ErrorKind::InvalidData, "Invalid Content-Length header")
                            })?
                            .trim();
                        content_length = length_str
                            .parse::<usize>()
                            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                        break;
                    }
                }

                // If there is a body, calculate remaining bytes to read, given that "\r\n\r\n" is
                // 4 bytes. We only need this clause for when the body is split across streams.
                // When the body is entirely in the next stream read, then `buffer.len() ==
                // body_start_pos`.
                let body_start_pos = headers_end_pos + 4;
                if buffer.len() > body_start_pos {
                    content_length -= buffer.len() - body_start_pos;
                }
            }
        }

        // If there is a body, read the remaining bytes. We do this in a loop in case it is sent
        // across multiple streams.
        while content_length > 0 {
            let num_bytes_read = self.stream.read(&mut temp_buffer).await?;
            if num_bytes_read == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Zero bytes read."));
            }
            buffer.extend_from_slice(&temp_buffer[..num_bytes_read]);
            content_length -= num_bytes_read;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::http::Method;

    impl<T: AsyncRead + AsyncWrite + Unpin> Connection<T> {
        /// In test contexts only, provide a reference to the underlying stream.
        fn stream(&self) -> &T {
            &self.stream
        }
    }

    #[tokio::test]
    async fn test_read_request_valid_root() {
        let stream = Cursor::new(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n".to_vec());
        let mut conn = Connection::new(stream);
        let request = conn.read_request().await.unwrap();
        assert_eq!(
            request,
            Request::with_headers(
                Method::Get,
                "/",
                vec![("Host".to_string(), "localhost".to_string())]
            )
        );
    }

    #[tokio::test]
    async fn test_read_request_invalid_utf8() {
        let stream = Cursor::new(b"\x80\x81\x82\x83".to_vec());
        let mut conn = Connection::new(stream);
        let result = conn.read_request().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);

        let stream = Cursor::new(b"\x80\x81\x82\x83\r\n\r\n".to_vec());
        let mut conn = Connection::new(stream);
        let result = conn.read_request().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn test_read_request_empty() {
        let stream = Cursor::new(b"".to_vec());
        let mut conn = Connection::new(stream);
        let result = conn.read_request().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn test_read_request_invalid_format() {
        let stream = Cursor::new(b"INVALID REQUEST\r\n".to_vec());
        let mut conn = Connection::new(stream);
        let result = conn.read_request().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);

        let stream = Cursor::new(b"INVALID REQUEST\r\n\r\n".to_vec());
        let mut conn = Connection::new(stream);
        let result = conn.read_request().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn test_send_response_ok() {
        let response = Response::new(
            200,
            vec![("Content-Type".to_string(), "text/plain".to_string())],
            "Hello, World!".to_string(),
        );

        let stream = Cursor::new(vec![]);
        let mut conn = Connection::new(stream);
        let result = conn.send_response(response).await;
        assert!(result.is_ok());
        assert_eq!(
            conn.stream().clone().into_inner(),
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!"
        );
    }

    #[tokio::test]
    async fn test_send_response_io_error() {
        use std::{
            io::{Error, ErrorKind, Result},
            pin::Pin,
            task::{Context, Poll},
        };
        use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

        struct FailingWriter;

        impl AsyncWrite for FailingWriter {
            fn poll_write(
                self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
                _buf: &[u8],
            ) -> Poll<Result<usize>> {
                Poll::Ready(Err(Error::new(ErrorKind::Other, "write failed")))
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
                Poll::Ready(Err(Error::new(ErrorKind::Other, "flush failed")))
            }

            fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        impl AsyncRead for FailingWriter {
            fn poll_read(
                self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
                _buf: &mut ReadBuf<'_>,
            ) -> Poll<Result<()>> {
                unimplemented!("We don't test this here.")
            }
        }

        let response = Response::new(200, vec![], "Hello, World!".to_string());
        let stream = FailingWriter;
        let mut conn = Connection::new(stream);

        let result = conn.send_response(response).await;
        assert!(result.is_err());
    }
}
