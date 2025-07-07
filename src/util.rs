use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};

/// Creates a `BoxBody` containing nothing.
/// 
/// # Example
/// 
/// ```
/// use hyper::Response;
/// use radmin_router::empty;
/// 
/// Response::builder()
///     .status(204)
///     .body(empty())
///     .unwrap();
pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::new()
        .map_err(|e| match e {})
        .boxed()
}

/// Creates a `BoxBody` from a `Bytes`-convertible body.
/// 
/// # Example
/// 
/// ```
/// use hyper::Response;
/// use radmin_router::full;
///
/// Response::builder()
///     .status(200)
///     .body(full("OK"))
///     .unwrap();
pub fn full<I: Into<Bytes>>(body: I) -> BoxBody<Bytes, hyper::Error> {
    Full::new(body.into())
        .map_err(|e| match e {})
        .boxed()
}