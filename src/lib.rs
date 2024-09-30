//! An HTTP web framework inspired by `axum` to learn intermediate Rust concepts.

#![warn(clippy::unwrap_used)]
#![warn(rust_2018_idioms)]

mod core;
mod handler;
mod into_response;
mod path_router;
mod router;
mod server;

pub use router::Router;
pub use server::serve;
pub mod extract;
pub mod http;

pub mod routing {
    //! Routing from requests to handlers
    use super::*;

    pub use path_router::{delete, get, head, options, patch, post, put};
}

pub mod response {
    //! Utilities for generating responses
    use super::*;

    pub use into_response::IntoResponse;
}
