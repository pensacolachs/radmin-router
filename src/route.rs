use crate::context::Context;
use crate::path::Path;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::{Method, Request};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;

/// The standard return type for all handlers. Returned to hyper.
pub type Response = Result<hyper::Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>;
/// The return type of async request handlers.
pub type ResponseFut = dyn Future<Output = Response> + Send + 'static;

/// A function pointer type for HTTP request handlers.
pub type Handler<Extra> = fn(Request<Incoming>, Context<Extra>) -> Pin<Box<ResponseFut>>;

/// A route representing a single endpoint (including all matching dynamic segments and HTTP methods).
pub struct Route<Extra: Send + Sync> {
    pub path: Path,
    handlers: HashMap<Method, Handler<Extra>>,
}

impl<Extra: Send + Sync> Route<Extra> {
    /// Constructs a new `Route<Extra>` with the provided path.
    /// 
    /// # Example
    /// 
    /// ```
    /// use radmin_router::{path, Route};
    /// Route::<()>::new(path!("/path/[to]/resource"));
    /// ```
    pub fn new(path: impl Into<Path>) -> Self {
        Self {
            path: path.into(),
            handlers: Default::default(),
        }
    }

    /// Returns the methods for which this route has registered handlers.
    pub fn allowed_methods(&self) -> Vec<Method> {
        self.handlers.keys().map(|m| m.clone()).collect()
    }

    pub(crate) fn handler(&self, method: &Method) -> Option<Handler<Extra>> {
        self.handlers.get(method).cloned()
    }

    fn register(mut self, method: Method, handler: Handler<Extra>) -> Self {
        self.handlers.insert(method, handler);
        self
    }

    /// Registers a handler for GET requests.
    pub fn get(self, handler: Handler<Extra>) -> Self {
        self.register(Method::GET, handler)
    }

    /// Registers a handler for POST requests.
    pub fn post(self, handler: Handler<Extra>) -> Self {
        self.register(Method::POST, handler)
    }

    /// Registers a handler for PUT requests.
    pub fn put(self, handler: Handler<Extra>) -> Self {
        self.register(Method::PUT, handler)
    }

    /// Registers a handler for DELETE requests.
    pub fn delete(self, handler: Handler<Extra>) -> Self {
        self.register(Method::DELETE, handler)
    }

    /// Registers a handler for HEAD requests.
    pub fn head(self, handler: Handler<Extra>) -> Self {
        self.register(Method::HEAD, handler)
    }

    /// Registers a handler for OPTIONS requests.
    pub fn options(self, handler: Handler<Extra>) -> Self {
        self.register(Method::OPTIONS, handler)
    }

    /// Registers a handler for CONNECT requests.
    pub fn connect(self, handler: Handler<Extra>) -> Self {
        self.register(Method::CONNECT, handler)
    }

    /// Registers a handler for PATCH requests.
    pub fn patch(self, handler: Handler<Extra>) -> Self {
        self.register(Method::PATCH, handler)
    }

    /// Registers a handler for TRACE requests.
    pub fn trace(self, handler: Handler<Extra>) -> Self {
        self.register(Method::TRACE, handler)
    }
}

impl<Extra: Send + Sync> Clone for Route<Extra> {
    fn clone(&self) -> Self {
        Self {
            path: Clone::clone(&self.path),
            handlers: Clone::clone(&self.handlers),
        }
    }
}

impl<Extra: Send + Sync> Debug for Route<Extra> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_tuple("Route");

        for method in self.handlers.keys() {
            debug.field(method);
        }

        debug.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_methods() {
        let mut route = Route::<()>::new(vec![]);
        route = route
            .get(|_, _| unimplemented!())
            .patch(|_, _| unimplemented!());

        let mut allowed_methods = route.allowed_methods()
            .into_iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>();
        allowed_methods.sort();
        let allowed_methods = allowed_methods.join(", ");
        
        assert_eq!(allowed_methods, "GET, PATCH");
    }

    #[test]
    fn register_handler() {
        let mut route = Route::<()>::new(vec![]);
        assert!(route.handler(&Method::GET).is_none());

        route = route.get(|_, _| unimplemented!());
        assert!(route.handler(&Method::GET).is_some());
    }
}
