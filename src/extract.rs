use crate::{
    http::{Request, Response},
    response::IntoResponse,
};

/// This represents a placeholder error type for when an extractor fails. For now, we will
/// universally consider this to be a 400 BAD REQUEST.
#[derive(Debug)]
pub struct ExtractError;

impl IntoResponse for ExtractError {
    fn into_response(self) -> Response {
        Response::new(
            400,
            vec![("Content-Type".to_string(), "text/plain".to_string())],
            "Bad request".to_string(),
        )
    }
}

/// A type that can be constructed ("extracted") from an incoming [`Request`].
///
/// Each parameter in a handler function must implement this trait in order to receive data from
/// the request—whether it comes from the path, headers, or body.
///
/// In this simplified version of the framework, we pass an immutable reference to the [`Request`]
/// into each extractor. Extractors that need to read the request body (for example, a `String`
/// extractor) simply clone it for demonstration purposes.
///
/// In real frameworks, extractors that *consume* the body would take ownership of the [`Request`]
/// instead, and only one such extractor could appear per handler.
pub trait FromRequest: Sized {
    fn from_request(req: &Request) -> Result<Self, ExtractError>;
}

/// Represents parameters of type `T` we expect to parse from the path. The data `T` must be public
/// for destructuring to work in the handler function signatures.
pub struct Path<T>(pub T);

impl FromRequest for Path<usize> {
    /// For simplicity, Cairo only supports a single path parameter per route.
    /// Frameworks like Axum support multiple (e.g. /users/:id/posts/:post_id) by storing all
    /// params in a map and deserializing them with serde.
    fn from_request(req: &Request) -> Result<Self, ExtractError> {
        let parts = req.into_parts();
        let param = parts
            .path_params
            .first()
            .ok_or(ExtractError)?
            .parse()
            .map_err(|_| ExtractError)?;
        Ok(Self(param))
    }
}

impl FromRequest for String {
    /// Extracts the entire request body as a `String`.
    ///
    /// In this teaching implementation, the body is cloned rather than consumed so that
    /// other extractors can still inspect the same request. Real web frameworks would consume the
    /// body here instead. I'm sorry if you thought this was a real web framework.
    fn from_request(req: &Request) -> Result<Self, ExtractError> {
        req.body.clone().ok_or(ExtractError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::http::Method;

    #[test]
    fn test_from_request_parts() {
        struct DummyExtractor;
        impl FromRequest for DummyExtractor {
            fn from_request(_req: &Request) -> Result<Self, ExtractError> {
                Ok(DummyExtractor)
            }
        }

        let req = Request::new(Method::Get, "/42");
        let extractor = DummyExtractor::from_request(&req).expect("Should return extractor");
        assert!(matches!(extractor, DummyExtractor));
    }

    #[test]
    fn test_from_request_path_usize() {
        let mut req = Request::new(Method::Get, "/42");
        req.set_path_params(vec!["42".to_string()]);
        let path: Path<usize> = Path::from_request(&req).expect("Should parse path param.");
        assert_eq!(path.0, 42);
    }

    #[test]
    #[should_panic]
    fn test_from_request_path_usize_invalid() {
        let req = Request::new(Method::Get, "/");
        let _path: Path<usize> = Path::from_request(&req).expect("This to fail");
    }
}
