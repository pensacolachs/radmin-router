use bytes::Bytes;
use http_body_util::combinators::BoxBody;

mod context;
mod node;
mod path;
mod route;
mod router;
mod segment;
#[cfg(feature = "util")]
mod util;

pub use context::*;
pub use macros;
pub use path::*;
pub use route::*;
pub use router::*;
pub use segment::*;
#[cfg(feature = "util")]
pub use util::*;

pub type Response = hyper::Response<BoxBody<Bytes, hyper::Error>>;
pub type Result = std::result::Result<Response, hyper::Error>;
