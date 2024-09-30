use std::collections::HashMap;

use crate::{
    handler::{BoxedHandler, Handler},
    http::Method,
};

/// A function called by [`add_http_function`] to start a chain of method/handler pairs by creating
/// a new [`PathRouter`].
fn on<H, T>(method: Method, handler: H) -> PathRouter
where
    H: Handler<T> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    PathRouter::new().on(method, handler)
}

/// A macro to allow a [`Method`] + [`Handler`] to start a chain of method/handler pairs.
macro_rules! add_http_function {
    (
        $name:ident, $method:ident
    ) => {
        #[doc = concat!("Register `", stringify!($method) ,"` requests to the given handler.")]
        pub fn $name<H, T>(handler: H) -> PathRouter
        where
            H: Handler<T> + Send + Sync + 'static,
            T: Send + Sync + 'static,
        {
            on(Method::$method, handler)
        }
    };
}

/// A macro to allow a [`Method`] + [`Handler`] to be chained onto an existing [`PathRouter`].
macro_rules! add_http_method {
    (
        $name:ident, $method:ident
    ) => {
        pub fn $name<H, T>(self, handler: H) -> Self
        where
            H: Handler<T> + Send + Sync + 'static,
            T: Send + Sync + 'static,
        {
            self.on(Method::$method, handler)
        }
    };
}

add_http_function!(get, Get);
add_http_function!(post, Post);
add_http_function!(options, Options);
add_http_function!(head, Head);
add_http_function!(put, Put);
add_http_function!(delete, Delete);
add_http_function!(patch, Patch);

/// A struct which holds the registration of [`BoxedHandler`] objects whose types have been erased
/// and which HTTP REST [`Method`]s they correspond to. Each of these corresponds to a single URL
/// pattern. See [`crate::Router`] for where all the URL patterns for an app are defined.
pub struct PathRouter {
    routes: HashMap<Method, BoxedHandler>,
}

impl PathRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::default(),
        }
    }

    pub fn find(&self, method: &Method) -> Option<BoxedHandler> {
        self.routes.get(method).cloned()
    }

    add_http_method!(get, Get);
    add_http_method!(post, Post);
    add_http_method!(options, Options);
    add_http_method!(head, Head);
    add_http_method!(put, Put);
    add_http_method!(delete, Delete);
    add_http_method!(patch, Patch);

    /// A private method which is called from [`chained_handler`] to register a [`Handler`].
    fn on<H, T>(mut self, method: Method, handler: H) -> Self
    where
        H: Handler<T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.routes
            .insert(method, BoxedHandler::from_handler(handler));
        self
    }
}

impl Default for PathRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{http::Response, response::IntoResponse};

    use super::*;

    #[derive(Debug, PartialEq)]
    struct MockResponse;

    impl IntoResponse for MockResponse {
        fn into_response(self) -> Response {
            todo!();
        }
    }

    fn hello_world() -> impl IntoResponse {
        "Hello, world!".to_string()
    }

    #[test]
    fn test_path_router_new() {
        let router: PathRouter = PathRouter::new();
        assert!(router.routes.is_empty());
    }

    #[test]
    fn test_path_router_default() {
        let router: PathRouter = PathRouter::default();
        assert!(router.routes.is_empty());
    }

    #[test]
    fn test_path_router_get() {
        let router = get(|| MockResponse);
        let handler = router.find(&Method::Get);
        assert!(handler.is_some());

        let handler = router.find(&Method::Post);
        assert!(handler.is_none());
    }

    #[test]
    fn test_path_router_post() {
        let router = post(|| MockResponse);
        let handler = router.find(&Method::Post);
        assert!(handler.is_some());

        let handler = router.find(&Method::Get);
        assert!(handler.is_none());
    }

    #[test]
    fn test_path_router_find_none() {
        let router = PathRouter::new();
        let handler = router.find(&Method::Get);
        assert!(handler.is_none());
    }

    #[test]
    fn test_path_router_get_and_post() {
        // TODO we cannot use MockResponse here yet because two closures are treated as different
        // types and we do not yet handle that.
        let router = get(hello_world).post(hello_world);

        let get_handler = router.find(&Method::Get);
        assert!(get_handler.is_some());

        let post_handler = router.find(&Method::Post);
        assert!(post_handler.is_some());
    }

    #[test]
    fn test_path_router_post_and_get() {
        // TODO we cannot use MockResponse here yet because two closures are treated as different
        // types and we do not yet handle that.
        let router = post(hello_world).get(hello_world);

        let get_handler = router.find(&Method::Get);
        assert!(get_handler.is_some());

        let post_handler = router.find(&Method::Post);
        assert!(post_handler.is_some());
    }
}
