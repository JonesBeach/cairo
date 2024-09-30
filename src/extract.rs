use crate::{
    http::{Parts, Request, Response},
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

/// Any extractor which does not need the [`Request`] body should implement this trait. If the body
/// will be consumed, the extractor should implement [`FromRequest`] instead.
pub trait FromRequestParts: Sized {
    fn from_request_parts(parts: &Parts) -> Result<Self, ExtractError>;
}

/// Any extractor which will consume the [`Request`] body must implement this. Because this is a
/// destructive action, these extractors must be placed last in the list of parameters for a
/// handler.
///
/// Since it is possible that the last parameter in the list will not need to consume the body, it
/// is recommended to implement this type and invoke its associated implementation for all types
/// which implement [`FromRequestParts`].
pub trait FromRequest: Sized {
    fn from_request(req: Request) -> Result<Self, ExtractError>;
}

/// Represents parameters of type `T` we expect to parse from the path. The data `T` must be public
/// for destructuring to work in the handler function signatures.
pub struct Path<T>(pub T);

impl FromRequestParts for Path<usize> {
    /// Pull the `Path<usize>` from the parts.
    fn from_request_parts(parts: &Parts) -> Result<Self, ExtractError> {
        let param = parts
            .path_params
            .first()
            .ok_or(ExtractError)?
            .parse()
            .map_err(|_| ExtractError)?;
        Ok(Self(param))
    }
}

impl FromRequest for Path<usize> {
    /// When a `Path<usize>` is requested as the last parameter, we pull it from the parts like
    /// normal.
    fn from_request(req: Request) -> Result<Self, ExtractError> {
        let parts = req.into_parts();
        Self::from_request_parts(parts)
    }
}

impl FromRequest for String {
    /// A `String` as the last parameter of a handler indicates we should parse the request body as
    /// plain text.
    fn from_request(req: Request) -> Result<Self, ExtractError> {
        req.body.ok_or(ExtractError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::http::Method;

    #[test]
    fn test_from_request_parts() {
        struct DummyExtractor;
        impl FromRequestParts for DummyExtractor {
            fn from_request_parts(_parts: &Parts) -> Result<Self, ExtractError> {
                Ok(DummyExtractor)
            }
        }

        let parts = Parts {
            method: Method::Get,
            path: "/".to_string(),
            headers: vec![],
            path_params: vec!["dummy".to_string()],
        };
        let extractor =
            DummyExtractor::from_request_parts(&parts).expect("Should return extractor");
        assert!(matches!(extractor, DummyExtractor));
    }

    #[test]
    fn test_from_request_path_usize() {
        let mut req = Request::new(Method::Get, "/42");
        req.set_path_params(vec!["42".to_string()]);
        let path: Path<usize> = Path::from_request(req).expect("Should parse path param.");
        assert_eq!(path.0, 42);
    }

    #[test]
    #[should_panic]
    fn test_from_request_path_usize_invalid() {
        let req = Request::new(Method::Get, "/");
        let _path: Path<usize> = Path::from_request(req).expect("This to fail");
    }
}
