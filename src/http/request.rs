use std::{error, fmt};

use crate::http::Method;

/// At this time, we only support HTTP/1. `hyper` supports HTTP/2.
pub(super) const PROTOCOL: &str = "HTTP/1.1";

/// Type alias representing the key-value pairs from a `Request` headers.
pub type Headers = Vec<(String, String)>;

/// Type alias representing the path parameters which are parsed from a request. This is not known
/// until it is matched against a `Router` pattern; the initial `Request` parsing is unaware of
/// these.
pub type PathParams = Vec<String>;

/// Represents everything we can extract from a request, besides the body. These are cheaper to
/// consume than the body and are often treated separate.
#[derive(Debug, PartialEq)]
pub struct Parts {
    pub method: Method,
    pub path: String,
    pub headers: Headers,
    pub path_params: PathParams,
}

#[derive(Debug, PartialEq)]
/// Error type for an invalid `Request`.
pub struct InvalidRequestError;

impl fmt::Display for InvalidRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid HTTP request")
    }
}

impl error::Error for InvalidRequestError {}

/// Represents an HTTP request.
#[derive(Debug, PartialEq)]
pub struct Request {
    parts: Parts,
    pub body: Option<String>,
}

impl Request {
    /// Create a new `Request` instance with the given method and path.
    pub fn new(method: Method, path: &str) -> Self {
        Self::with_headers(method, path, Headers::default())
    }

    /// Create a new `Request` instance with the given method, path, and headers.
    pub fn with_headers(method: Method, path: &str, headers: Headers) -> Self {
        let parts = Parts {
            method,
            path: path.to_string(),
            headers,
            path_params: PathParams::default(),
        };

        Self { parts, body: None }
    }

    /// Set the path parameters for the request.
    pub fn set_path_params(&mut self, path_params: PathParams) {
        self.parts.path_params = path_params;
    }

    /// Set the path parameters for the request.
    pub fn set_headers(&mut self, headers: Headers) {
        self.parts.headers = headers;
    }

    /// Set the body for the request.
    pub fn set_body(&mut self, body: String) {
        self.body = Some(body);
    }

    /// Retrieve the path parameters, returning an empty vector if none are set.
    pub fn into_parts(&self) -> &Parts {
        &self.parts
    }

    /// `Method` accessor.
    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    /// Path accessor.
    pub fn path(&self) -> &String {
        &self.parts.path
    }

    /// `Headers` accessor.
    pub fn headers(&self) -> &Headers {
        &self.parts.headers
    }

    /// `PathParams` accessor.
    pub fn path_params(&self) -> &PathParams {
        &self.parts.path_params
    }
}

impl TryFrom<&str> for Request {
    type Error = InvalidRequestError;

    /// Parses a raw HTTP request string into a `Request` instance.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut lines = value.split("\r\n");

        let request_line = lines.next().ok_or(InvalidRequestError)?;
        let mut request_line_parts = request_line.split_whitespace();

        let method = request_line_parts
            .next()
            .ok_or(InvalidRequestError)?
            .try_into()
            .map_err(|_| InvalidRequestError)?;
        let path = request_line_parts.next().ok_or(InvalidRequestError)?;
        let version = request_line_parts.next().ok_or(InvalidRequestError)?;
        // We could move this into a Version enum
        if version == "HTTP/2.0" {
            unimplemented!("Unsupported HTTP version: {}", version);
        }

        // Confirm the request line has exactly 3 parts
        if request_line_parts.next().is_some() {
            return Err(InvalidRequestError);
        }

        let mut request = Request::new(method, path);

        let mut headers = vec![];
        for line in &mut lines {
            // An empty line indicates the end of the headers
            if line.is_empty() {
                break;
            }

            let mut header_parts = line.splitn(2, ": ");
            let name = header_parts.next().ok_or(InvalidRequestError)?.to_string();
            let value = header_parts.next().ok_or(InvalidRequestError)?.to_string();
            headers.push((name, value));
        }

        request.set_headers(headers);

        // The request body consists of the remaining lines, so we rejoin them
        let body = lines.collect::<Vec<&str>>().join("\r\n");
        if !body.is_empty() {
            request.set_body(body);
        }

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_valid_root() {
        let stream = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let request = Request::try_from(stream).unwrap();
        assert_eq!(
            request,
            Request::with_headers(
                Method::Get,
                "/",
                vec![("Host".to_string(), "localhost".to_string())]
            )
        );

        let stream = "POST / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let request = Request::try_from(stream).unwrap();
        assert_eq!(
            request,
            Request::with_headers(
                Method::Post,
                "/",
                vec![("Host".to_string(), "localhost".to_string())]
            )
        );
    }

    #[test]
    fn test_parse_request_invalid_method() {
        let stream = "PAST / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let request = Request::try_from(stream);
        assert!(request.is_err(), "Request should not be formed.");
    }

    #[test]
    fn test_parse_request_valid_path() {
        let stream = "GET /path HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let request = Request::try_from(stream).unwrap();
        assert_eq!(
            request,
            Request::with_headers(
                Method::Get,
                "/path",
                vec![("Host".to_string(), "localhost".to_string())]
            )
        );
    }

    #[test]
    fn test_parse_request_valid_post() {
        let stream = "POST /post/5 HTTP/1.1\r\n\
                       Host: 127.0.0.1:7878\r\n\
                       User-Agent: curl/8.4.0\r\n\
                       Accept: */*\r\n\
                       Content-Length: 10\r\n\
                       Content-Type: application/x-www-form-urlencoded\r\n\
                       \r\n\
                       Hello Rust";
        let request = Request::try_from(stream).unwrap();
        let mut expected = Request::with_headers(
            Method::Post,
            "/post/5",
            vec![
                ("Host".to_string(), "127.0.0.1:7878".to_string()),
                ("User-Agent".to_string(), "curl/8.4.0".to_string()),
                ("Accept".to_string(), "*/*".to_string()),
                ("Content-Length".to_string(), "10".to_string()),
                (
                    "Content-Type".to_string(),
                    "application/x-www-form-urlencoded".to_string(),
                ),
            ],
        );
        expected.set_body("Hello Rust".to_string());

        assert_eq!(request, expected);
    }

    #[test]
    fn test_parse_request_empty() {
        let stream = "";
        let result = Request::try_from(stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_request_whitespace_only() {
        let stream = "   \r\n   ";
        let result = Request::try_from(stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_request_whitespace_only_single_line() {
        let stream = "      ";
        let result = Request::try_from(stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_request_invalid_format() {
        let stream = "INVALID REQUEST\r\n";
        let result = Request::try_from(stream);
        assert!(result.is_err());
    }
}
