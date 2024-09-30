use std::{error, fmt};

/// Enumeration representing HTTP methods.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Options,
    Head,
    Put,
    Delete,
    Patch,
}

/// Error type for invalid HTTP methods.
#[derive(Debug, PartialEq)]
pub struct InvalidMethodError;

impl fmt::Display for InvalidMethodError {
    /// The `'_` lifetime in the `fmt::Formatter<'_>` parameter tells the Rust compiler that the
    /// `Formatter` reference must live at least as long as the `fmt` method call. This is a
    /// shorthand for using a named lifetime, such as `'a`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid HTTP method")
    }
}

/// This isn't technically required, but having our error implement `std::error::Error` is
/// idiomatic and will allow our error type to interface with other future libraries better.
impl error::Error for InvalidMethodError {}

impl TryFrom<&str> for Method {
    type Error = InvalidMethodError;

    /// Attempt to convert a string slice to a `Method` enum.
    /// Return `Ok(Method)` if the string is a valid HTTP method, otherwise returns
    /// `Err(InvalidMethodError)`.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "OPTIONS" => Ok(Method::Options),
            "HEAD" => Ok(Method::Head),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            _ => Err(InvalidMethodError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_option_get() {
        let method = "GET".try_into();
        assert_eq!(method, Ok(Method::Get));
    }

    #[test]
    fn test_try_into_option_post() {
        let method = "POST".try_into();
        assert_eq!(method, Ok(Method::Post));
    }

    #[test]
    fn test_try_into_option_options() {
        let method = "OPTIONS".try_into();
        assert_eq!(method, Ok(Method::Options));
    }

    #[test]
    fn test_try_into_option_head() {
        let method = "HEAD".try_into();
        assert_eq!(method, Ok(Method::Head));
    }

    #[test]
    fn test_try_into_option_put() {
        let method = "PUT".try_into();
        assert_eq!(method, Ok(Method::Put));
    }

    #[test]
    fn test_try_into_option_delete() {
        let method = "DELETE".try_into();
        assert_eq!(method, Ok(Method::Delete));
    }

    #[test]
    fn test_try_into_option_patch() {
        let method = "PATCH".try_into();
        assert_eq!(method, Ok(Method::Patch));
    }

    #[test]
    fn test_try_into_option_invalid() {
        let method = Method::try_from("UNDELETE");
        assert_eq!(method, Err(InvalidMethodError));
    }
}
