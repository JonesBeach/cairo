use crate::http::Response;

/// Trait to convert a value into a `Response`.
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for &str {
    /// Convert a `&str` into a `Response`, by way of a `String`.
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}

impl IntoResponse for String {
    /// Convert a `String` into a `Response`, using its value as the body.
    fn into_response(self) -> Response {
        Response::new(
            200,
            vec![("Content-Type".to_string(), "text/plain".to_string())],
            self,
        )
    }
}

impl IntoResponse for (u16, &str) {
    /// Convert a `(u16, String)` into a `Response`, using its value as the HTTP response code and
    /// body.
    fn into_response(self) -> Response {
        Response::new(
            self.0,
            vec![("Content-Type".to_string(), "text/plain".to_string())],
            self.1.to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_into_response() {
        let body = "Hello, world!".to_string();
        let response: Response = body.clone().into_response();
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), body);
        assert_eq!(
            response.headers(),
            &vec![("Content-Type".to_string(), "text/plain".to_string())]
        );
    }

    #[test]
    fn test_tuple_into_response() {
        let status_code = 404;
        let body = "Page not found";
        let response: Response = (status_code, body).into_response();
        assert_eq!(response.status_code(), status_code);
        assert_eq!(response.text(), body);
        assert_eq!(
            response.headers(),
            &vec![("Content-Type".to_string(), "text/plain".to_string())]
        );
    }
}
