use std::{marker::PhantomData, sync::Arc};

use crate::{
    extract::FromRequest,
    http::{Request, Response},
    response::IntoResponse,
};

/// This trait allows us to call a [`Handler`] without knowing its concrete type.
trait ErasedHandler {
    fn call_handler(&self, req: Request) -> Response;
}

/// [`BoxedHandler`] wraps around an [`Arc`] of an [`ErasedHandler`], allowing it to be cloned and
/// used without knowing the concrete type.
pub struct BoxedHandler(Arc<dyn ErasedHandler + Send + Sync>);

impl BoxedHandler {
    /// Create a new [`BoxedHandler`] from a given [`Handler`].
    pub fn from_handler<H, T>(handler: H) -> Self
    where
        H: Handler<T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        Self(Arc::new(ConcreteHandler::new(handler)))
    }

    pub fn call_handler(&self, req: Request) -> Response {
        self.0.call_handler(req)
    }
}

impl Clone for BoxedHandler {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// A struct to hold a [`Handler`] which hides away its type info using [`PhantomData`].
struct ConcreteHandler<H, T> {
    handler: H,
    _marker: PhantomData<T>,
}

impl<H, T> ConcreteHandler<H, T> {
    fn new(handler: H) -> Self {
        Self {
            handler,
            _marker: PhantomData,
        }
    }
}

impl<H, T> ErasedHandler for ConcreteHandler<H, T>
where
    H: Handler<T> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    fn call_handler(&self, req: Request) -> Response {
        self.handler.call_handler(req)
    }
}

/// This trait allows us to take [`Request`] and returns a [`Response`]. At this
/// point, the handler's types are still known. We will erase them to use a [`ErasedHandler`].
pub trait Handler<T>: Sized {
    fn call_handler(&self, req: Request) -> Response;
}

/// Implement [`Handler`] for functions that take no arguments and return a [`Response`].
impl<T, R> Handler<((),)> for T
where
    T: Fn() -> R,
    R: IntoResponse,
{
    fn call_handler(&self, _req: Request) -> Response {
        self().into_response()
    }
}

/// Macro to implement [`Handler`] for functions that take various numbers of arguments.
macro_rules! handler_variable_args {
    (
        [$($ty:ident),*]
    ) => {
        #[allow(non_snake_case)]
        impl<T, R, $($ty,)*> Handler<($($ty,)*)> for T
        where
            T: Fn($($ty,)*) -> R,
            R: IntoResponse,
            $( $ty: FromRequest, )*
        {
            fn call_handler(&self, req: Request) -> Response {
                $(
                    let $ty = match $ty::from_request(&req) {
                        Ok(value) => value,
                        Err(e) => return e.into_response(),
                    };
                )*

                let res = self($($ty,)*);
                res.into_response()
            }
        }
    };
}

// Apply the handler for all the number of args we currently support.
handler_variable_args!([T1]);
handler_variable_args!([T1, T2]);
handler_variable_args!([T1, T2, T3]);
handler_variable_args!([T1, T2, T3, T4]);
handler_variable_args!([T1, T2, T3, T4, T5]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        extract::Path,
        http::{Method, Request},
    };

    #[test]
    fn test_handler_no_args() {
        fn handler() -> &'static str {
            "Hello, World!"
        }

        let boxed_handler = BoxedHandler::from_handler(handler);
        let req = Request::new(Method::Get, "/");
        let response = boxed_handler.call_handler(req);

        assert_eq!(response.text(), "Hello, World!");
    }

    #[test]
    fn test_handler_with_args() {
        fn handler(Path(id): Path<usize>) -> String {
            format!("ID: {}", id)
        }

        let boxed_handler = BoxedHandler::from_handler(handler);
        let mut req = Request::new(Method::Get, "/");
        req.set_path_params(vec!["42".to_string()]);
        let response = boxed_handler.call_handler(req);

        assert_eq!(response.text(), "ID: 42");
    }

    #[test]
    fn test_cloning_boxed_handler() {
        fn handler() -> &'static str {
            "Hello, World!"
        }

        let boxed_handler = BoxedHandler::from_handler(handler);
        let cloned_handler = boxed_handler.clone();
        let req = Request::new(Method::Get, "/");
        let response = cloned_handler.call_handler(req);

        assert_eq!(response.text(), "Hello, World!");
    }

    #[test]
    fn test_make_erased_handler() {
        fn handler() -> &'static str {
            "Hello, World!"
        }

        let erased_handler = ConcreteHandler {
            handler,
            _marker: PhantomData,
        };

        let req = Request::new(Method::Get, "/");
        let response = erased_handler.call_handler(req);

        assert_eq!(response.text(), "Hello, World!");
    }
}
