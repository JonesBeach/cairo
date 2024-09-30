use std::collections::HashMap;

use crate::{
    http::{PathParams, Request, Response},
    path_router::PathRouter,
    response::IntoResponse,
};

/// Function to match a route pattern with an actual path
fn match_route(route_pattern: &str, path: &str) -> Option<PathParams> {
    let extract_segments = |path: &str| {
        path.trim_start_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    let route_segments = extract_segments(route_pattern);
    let path_segments = extract_segments(path);

    if route_segments.len() != path_segments.len() {
        return None;
    }

    // Collect parameters from the path if the segments match
    let mut params = vec![];
    for (route_segment, path_segment) in route_segments.iter().zip(path_segments.iter()) {
        if route_segment.starts_with(':') {
            params.push(path_segment.to_string());
        } else if route_segment != path_segment {
            return None;
        }
    }

    Some(params)
}

/// Router struct to manage routes and handlers
pub struct Router {
    routes: HashMap<String, PathRouter>,
}

impl Router {
    /// Create a new `Router` instance
    pub fn new() -> Self {
        Self {
            routes: HashMap::default(),
        }
    }

    /// Add a route with its handler to the router
    pub fn route(mut self, path: &str, handler: PathRouter) -> Self {
        self.routes.insert(path.to_string(), handler);
        self
    }

    /// Call the appropriate handler based on the request
    pub(crate) fn call(&self, mut request: Request) -> Response {
        let mut found_path_params = None;
        let handler = self.routes.iter().find_map(|(pattern, path_router)| {
            match match_route(pattern, request.path()) {
                Some(path_params) => {
                    found_path_params = Some(path_params);
                    path_router.find(request.method())
                }
                None => None,
            }
        });

        if let Some(path_params) = found_path_params {
            request.set_path_params(path_params);
        }

        match handler {
            Some(handler) => handler.call_handler(request),
            None => (404, "Not Found").into_response(),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{extract::Path, http::Method, routing::get};

    use super::*;

    fn hello_world() -> impl IntoResponse {
        "Hello, world!".to_string()
    }

    fn hello_world_index(Path(id): Path<usize>) -> impl IntoResponse {
        format!("Hello, world: {}!", id)
    }

    #[test]
    fn test_router_new() {
        let router: Router = Router::new();
        assert!(
            router.routes.is_empty(),
            "Routes should be empty on creation"
        );
    }

    #[test]
    fn test_router_route() {
        let router = Router::new().route("/hello", get(hello_world));

        let req = Request::new(Method::Get, "/hello");
        let response = router.call(req);
        assert_eq!(
            response.text(),
            "Hello, world!",
            "Handler should return 'Hello, world!'"
        );

        let response = router.call(Request::new(Method::Get, "/goodbye"));
        assert_eq!(
            response.text(),
            "Not Found",
            "Handler should return 'Not Found'"
        );

        let response = router.call(Request::new(Method::Post, "/hello"));
        assert_eq!(
            response.text(),
            "Not Found",
            "Handler should return 'Not Found'"
        );
    }

    #[test]
    fn test_router_route_multiple_handlers() {
        let router = Router::new().route("/hello", get(hello_world).post(hello_world));

        let req = Request::new(Method::Get, "/hello");
        let response = router.call(req);
        assert_eq!(
            response.text(),
            "Hello, world!",
            "Handler should return 'Hello, world!'"
        );

        let req = Request::new(Method::Post, "/hello");
        let response = router.call(req);
        assert_eq!(
            response.text(),
            "Hello, world!",
            "Handler should return 'Hello, world!'"
        );
    }

    #[test]
    fn test_router_route_with_arg() {
        let router = Router::new().route("/hello/:id", get(hello_world_index));

        let req = Request::new(Method::Get, "/hello/5");
        let response = router.call(req);
        assert_eq!(
            response.text(),
            "Hello, world: 5!",
            "Handler should return 'Hello, world: 5!'"
        );
    }

    #[test]
    fn test_router_route_with_multiple_routes() {
        let router = Router::new()
            .route("/hello/:id", get(hello_world_index))
            .route("/hello", get(hello_world));

        let req = Request::new(Method::Get, "/hello/5");
        let response = router.call(req);
        assert_eq!(
            response.text(),
            "Hello, world: 5!",
            "Handler should return 'Hello, world: 5!'"
        );

        let req = Request::new(Method::Get, "/hello");
        let response = router.call(req);
        assert_eq!(
            response.text(),
            "Hello, world!",
            "Handler should return 'Hello, world!'"
        );
    }

    #[test]
    fn test_router_default() {
        let router: Router = Default::default();
        assert!(
            router.routes.is_empty(),
            "Routes should be empty when using default"
        );
    }
}
