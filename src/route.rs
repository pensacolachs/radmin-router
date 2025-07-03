use std::collections::HashMap;
use crate::context::Context;
use crate::path::Path;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::{Method, Request};
use std::fmt::{Debug, Formatter};
use std::pin::Pin;

pub type Response = Result<hyper::Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>;
pub type ResponseFut = dyn Future<Output = Response> + Send + 'static;

pub type Handler<Extra> = fn(Request<Incoming>, Context<Extra>) -> Pin<Box<ResponseFut>>;

#[derive(Clone)]
pub struct Route<Extra: Clone + Send + Sync> {
    pub path: Path,
    handlers: HashMap<Method, Handler<Extra>>,
}

impl<Extra: Clone + Send + Sync> Route<Extra> {
    pub fn new(path: impl Into<Path>) -> Self {
        Self {
            path: path.into(),
            handlers: Default::default()
        }
    }

    pub fn allowed_methods(&self) -> Vec<Method> {
        self.handlers.keys()
            .map(|m| m.clone())
            .collect()
    }

    pub(crate) fn handler(&self, method: &Method) -> Option<Handler<Extra>> {
        self.handlers.get(method).cloned()
    }

    fn register(&mut self, method: Method, handler: Handler<Extra>) -> &mut Self {
        self.handlers.insert(method, handler);
        self
    }

    pub fn get(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::GET, handler)
    }

    pub fn post(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::POST, handler)
    }

    pub fn put(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::PUT, handler)
    }

    pub fn delete(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::DELETE, handler)
    }

    pub fn head(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::HEAD, handler)
    }

    pub fn options(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::OPTIONS, handler)
    }

    pub fn connect(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::CONNECT, handler)
    }

    pub fn patch(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::PATCH, handler)
    }

    pub fn trace(&mut self, handler: Handler<Extra>) -> &mut Self {
        self.register(Method::TRACE, handler)
    }
}

impl<Extra: Clone + Send + Sync> Debug for Route<Extra> {
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
        route
            .get(|_, _| unimplemented!())
            .patch(|_, _| unimplemented!());

        assert_eq!(route.allowed_methods(), vec![Method::GET, Method::PATCH]);
    }
    
    #[test]
    fn register_handler() {
        let mut route = Route::<()>::new(vec![]);
        assert!(route.handler(&Method::GET).is_none());
        
        route.get(|_, _| unimplemented!());
        assert!(route.handler(&Method::GET).is_some());
    }
}