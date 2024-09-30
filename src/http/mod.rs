//! Utilities for manipulating common objects. In a production system, the `http` crate should be
//! used instead.
mod method;
mod request;
mod response;

pub use method::Method;
pub use request::{Parts, PathParams, Request};
pub use response::Response;
