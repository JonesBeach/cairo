use super::request::{Headers, PROTOCOL};

/// Alias to represent the 3-digit HTTP status code. This will fall between 100 and 599, inclusive.
type StatusCode = u16;

/// Convert an HTTP status code to its corresponding reason phrase as a string.
fn status_code_to_string(code: StatusCode) -> &'static str {
    match code {
        200 => "OK",
        400 => "BAD REQUEST",
        404 => "NOT FOUND",
        _ => unimplemented!("Unsupported status code: {}", code),
    }
}

/// Represents an HTTP response.
pub struct Response {
    status_code: StatusCode,
    headers: Headers,
    body: String,
}

impl Response {
    /// Create a new `Response` instance with the given status code, headers, and body.
    pub fn new(status_code: StatusCode, headers: Headers, body: String) -> Self {
        Self {
            status_code,
            headers,
            body,
        }
    }

    /// Return the response body as a string.
    pub fn text(&self) -> String {
        self.body.clone()
    }

    /// Return the status code.
    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    /// Return the headers.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Return the entire HTTP response as a vector of bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.stream().into_bytes()
    }

    /// Construct the HTTP response as a string, including the status line, headers, and body.
    fn stream(&self) -> String {
        let headers = self
            .headers
            .iter()
            .fold(String::new(), |mut acc, (key, value)| {
                acc.push_str(&format!("{}: {}\r\n", key, value));
                acc
            });

        format!(
            "{} {} {}\r\n{}\r\n{}",
            PROTOCOL,
            self.status_code,
            status_code_to_string(self.status_code),
            headers,
            self.body,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_new() {
        let headers = vec![("Content-Type".to_string(), "text/plain".to_string())];
        let response = Response::new(200, headers.clone(), "Hello, World!".to_string());
        assert_eq!(response.status_code, 200);
        assert_eq!(response.headers, headers);
        assert_eq!(response.body, "Hello, World!".to_string());
    }

    #[test]
    fn test_response_text() {
        let response = Response::new(200, vec![], "Hello, World!".to_string());
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "Hello, World!".to_string());
        assert_eq!(response.headers(), &vec![]);
    }

    #[test]
    fn test_response_as_bytes() {
        let response = Response::new(200, vec![], "Hello, World!".to_string());
        let expected = "HTTP/1.1 200 OK\r\n\r\nHello, World!".as_bytes().to_vec();
        assert_eq!(response.as_bytes(), expected);
    }

    #[test]
    fn test_response_stream() {
        let response = Response::new(
            200,
            vec![("Content-Type".to_string(), "text/plain".to_string())],
            "Hello, World!".to_string(),
        );
        let expected =
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!".to_string();
        assert_eq!(response.stream(), expected);
    }

    #[test]
    fn test_status_code_to_string_200() {
        assert_eq!(status_code_to_string(200), "OK");
    }

    #[test]
    fn test_status_code_to_string_404() {
        assert_eq!(status_code_to_string(404), "NOT FOUND");
    }

    #[test]
    #[should_panic]
    fn test_status_code_to_string_unimplemented() {
        status_code_to_string(500);
    }
}
