use std::collections::HashMap;

use crate::{
    http::{PathParams, Request, Response},
    path_router::PathRouter,
    response::IntoResponse,
};

fn split_path(path: &str) -> Vec<&str> {
    path.trim_start_matches('/').split('/').collect()
}

fn route_matches(route_pattern: &str, path: &str) -> bool {
    let route_segments = split_path(route_pattern);
    let path_segments = split_path(path);

    if route_segments.len() != path_segments.len() {
        return false;
    }

    for (route_segment, path_segment) in route_segments.iter().zip(path_segments.iter()) {
        if !route_segment.starts_with(':') && route_segment != path_segment {
            return false;
        }
    }

    true
}

fn extract_path_params(route_pattern: &str, path: &str) -> PathParams {
    let route_segments = split_path(route_pattern);
    let path_segments = split_path(path);

    route_segments
        .iter()
        .zip(path_segments.iter())
        .filter_map(|(r, p)| r.strip_prefix(':').map(|_| p.to_string()))
        .collect()
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
        let path = request.path();
        let method = request.method();

        // Step 1: find the matching route pattern and its path router
        let Some((pattern, path_router)) = self
            .routes
            .iter()
            .find(|(pattern, _)| route_matches(pattern, path))
        else {
            return (404, "Not Found").into_response();
        };

        // Step 2: find the handler for the given HTTP method
        let Some(handler) = path_router.find(method) else {
            return (405, "Method Not Allowed").into_response();
        };

        // Step 3: extract parameters
        let params = extract_path_params(pattern, path);
        request.set_path_params(params);

        handler.call_handler(request)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod router_tests {
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
            "Method Not Allowed",
            "Handler should return 'Method Not Allowed'"
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

#[cfg(test)]
mod route_matches_tests {
    use super::*;

    #[test]
    fn matches_exact_path() {
        assert!(route_matches("/users", "/users"));
        assert!(route_matches("/users/list", "/users/list"));
    }

    #[test]
    fn matches_with_param() {
        assert!(route_matches("/users/:id", "/users/42"));
        assert!(route_matches(
            "/posts/:slug/comments/:cid",
            "/posts/hello-world/comments/123"
        ));
    }

    #[test]
    fn rejects_with_different_segment_counts() {
        assert!(!route_matches("/users/:id", "/users"));
        assert!(!route_matches("/users/:id", "/users/42/comments"));
    }

    #[test]
    fn rejects_with_literal_mismatch() {
        assert!(!route_matches("/users/:id", "/accounts/42"));
        assert!(!route_matches(
            "/posts/:slug/comments/:cid",
            "/posts/hello/likes/123"
        ));
    }

    #[test]
    fn allows_multiple_params() {
        assert!(route_matches("/a/:b/:c", "/a/1/2"));
        assert!(!route_matches("/a/:b/:c", "/a/1"));
    }
}

#[cfg(test)]
mod extract_path_params_tests {
    use super::*;

    #[test]
    fn extracts_single_param() {
        let params = extract_path_params("/users/:id", "/users/42");
        assert_eq!(params, vec!["42"]);
    }

    #[test]
    fn extracts_multiple_params() {
        let params = extract_path_params("/posts/:slug/comments/:cid", "/posts/hello/comments/99");
        assert_eq!(params, vec!["hello", "99"]);
    }

    #[test]
    fn extracts_none_when_no_params() {
        let params = extract_path_params("/users", "/users");
        assert!(params.is_empty());
    }

    #[test]
    fn ignores_literal_mismatches() {
        // Even though the literals don't match, extraction shouldn't panic.
        let params = extract_path_params("/users/:id", "/posts/42");
        assert_eq!(params, vec!["42"]);
    }

    #[test]
    fn ignores_extra_segments() {
        let params = extract_path_params("/users/:id", "/users/42/extra");
        assert_eq!(params, vec!["42"]);
    }
}
