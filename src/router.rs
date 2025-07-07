use crate::context::Context;
use crate::node::Node;
use crate::route::Route;
use crate::segment::Segment;
use bytes::Bytes;
use futures::future::BoxFuture;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode, header};
use std::fmt::Debug;
use std::sync::Arc;

#[cfg(feature = "logging")]
use std::time::Instant;

type RouteNotFoundHandler<Extra> =
    fn(Request<Incoming>, Arc<Extra>) -> BoxFuture<'static, crate::Result>;
type MethodNotAllowedHandler<Extra> =
    fn(Route<Extra>, Request<Incoming>, Context<Extra>) -> BoxFuture<'static, crate::Result>;

#[derive(Debug)]
pub struct Router<Extra: Send + Sync> {
    ex: Arc<Extra>,
    root: Node<Extra>,
    route_not_found: RouteNotFoundHandler<Extra>,
    method_not_allowed: MethodNotAllowedHandler<Extra>,
}

impl<Extra: Send + Sync> Clone for Router<Extra> {
    fn clone(&self) -> Self {
        Self {
            ex: Clone::clone(&self.ex),
            root: Clone::clone(&self.root),
            route_not_found: Clone::clone(&self.route_not_found),
            method_not_allowed: Clone::clone(&self.method_not_allowed),
        }
    }
}

impl<Extra: Default + Send + Sync> Default for Router<Extra> {
    fn default() -> Self {
        Self::new(Arc::new(Default::default()))
    }
}

impl<Extra: Send + Sync> Router<Extra> {
    pub fn new(ex: Arc<Extra>) -> Self {
        Self {
            ex,
            root: Node::default(),
            route_not_found: |_, _| {
                Box::pin(async {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(full("Not Found"))
                        .unwrap())
                })
            },
            method_not_allowed: |route, _, _| {
                let allowed_methods = route.allowed_methods()
                    .into_iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .header(header::ALLOW, allowed_methods)
                        .body(full("Method Not Allowed"))
                        .unwrap())
                })
            },
        }
    }

    /// Registers a handler to generate a response when no route is matched.
    ///
    /// # Examples
    ///
    /// ```
    /// use radmin_router::{Router, empty};
    /// use std::sync::Arc;
    /// use http_body_util::{BodyExt, Empty};
    /// use hyper::Response;
    ///
    /// Router::<()>::default()
    ///     .route_not_found(|req, ex| {
    ///         Box::pin(async move {
    ///             Ok(Response::builder()
    ///                 .status(404)
    ///                 .body(empty())
    ///                 .unwrap())
    ///         })
    ///     });
    /// ```
    pub fn route_not_found(&mut self, handler: RouteNotFoundHandler<Extra>) -> &mut Self {
        self.route_not_found = handler;
        self
    }

    /// Registers a handler to generate a response when a route without a handler
    /// for the requested method is matched.
    ///
    /// # Examples
    ///
    /// ```
    /// use radmin_router::{Router, full};
    /// use std::sync::Arc;
    /// use http_body_util::{BodyExt, Empty};
    /// use hyper::Response;
    ///
    /// Router::<()>::default()
    ///     .method_not_allowed(|route, req, ex| {
    ///         let methods = route.allowed_methods()
    ///             .into_iter()
    ///             .map(|m| m.to_string())
    ///             .collect::<Vec<_>>()
    ///             .join(", ");
    ///
    ///         Box::pin(async move {
    ///             Ok(Response::builder()
    ///                 .status(405)
    ///                 .header("Allow", methods)
    ///                 .body(full("Method Not Allowed"))
    ///                 .unwrap())
    ///         })
    ///     });
    /// ```
    pub fn method_not_allowed(&mut self, handler: MethodNotAllowedHandler<Extra>) -> &mut Self {
        self.method_not_allowed = handler;
        self
    }

    /// Registers a route, replacing an existing route with an equivalent path.
    ///
    /// # Example
    ///
    /// ```
    /// use hyper::Response;
    /// use radmin_router::{full, path, Route, Router};
    ///
    /// let route = Route::new(path!("/"))
    ///     .get(|req, ctx| {
    ///     Box::pin(async move {
    ///         Ok(Response::builder()
    ///             .status(200)
    ///             .body(full("OK"))
    ///             .unwrap())
    ///     })
    /// });
    ///
    /// Router::<()>::default()
    ///     .register(route);
    pub fn register(mut self, route: Route<Extra>) -> Self {
        self.root.append(route);
        self
    }

    pub fn register_many(&mut self, routes: impl IntoIterator<Item = Route<Extra>>) -> &mut Self {
        for route in routes {
            println!("Added route: {}", route.path);
            self.root.append(route);
        }

        self
    }

    fn match_route(&self, path: impl AsRef<str>) -> Option<(Route<Extra>, Vec<String>)> {
        let segments = path
            .as_ref()
            .trim_start_matches('/')
            .split('/')
            .collect::<Vec<_>>();

        let mut candidates = vec![&self.root];

        for segment in segments.iter() {
            if *segment == "" {
                continue;
            }

            let mut new_candidates = vec![];
            for candidate in candidates {
                if let Some(literal) = candidate.children.get(&Segment::literal(*segment)) {
                    new_candidates.push(literal);
                }

                if let Some(dynamic) = candidate.children.get(&Segment::dynamic("")) {
                    new_candidates.push(dynamic);
                }
            }

            if new_candidates.is_empty() {
                return None;
            }
            candidates = new_candidates;
        }

        if candidates.len() > 1 {
            eprintln!("Matched multiple routes! {:?}", candidates);
        }

        let route = candidates.first()?.route.as_ref()?;
        let params = route
            .path
            .0
            .iter()
            .enumerate()
            .fold(vec![], |mut acc, (idx, seg)| {
                if let Segment::Dynamic(_) = seg {
                    acc.push(segments[idx].to_string());
                }

                acc
            });

        Some((route.clone(), params))
    }

    /// Processes an incoming request and generates a response for hyper.
    pub async fn route(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        #[cfg(feature = "logging")]
        let before = Instant::now();
        #[cfg(feature = "logging")]
        let method = req.method().clone();

        let path = req.uri().path().to_string();

        let Some((route, params)) = self.match_route(&path) else {
            return (self.route_not_found)(req, Arc::clone(&self.ex)).await;
        };

        let ctx = Context {
            params,
            ex: Arc::clone(&self.ex),
        };

        let Some(handler) = route.handler(req.method()) else {
            return (self.method_not_allowed)(route, req, ctx).await;
        };

        let resp = handler(req, ctx).await;

        #[cfg(feature = "logging")]
        {
            use chrono::Utc;

            let elapsed = before.elapsed();
            match resp {
                Ok(ref resp) => {
                    let status_code = resp.status().as_u16();
                    let status_color = match status_code {
                        200..=299 => 92, // bright green
                        300..=399 => 95, // bright magenta
                        400..=499 => 93, // bright yellow
                        500..=599 => 91, // bright red
                        _ => 97,         // white
                    };

                    println!(
                        "\x1B[34m[{}] \x1B[{status_color}m{}\x1B[97m {:6} {} \x1B[37m({:?})",
                        Utc::now().format("%Y-%m-%d %H:%M:%S"),
                        status_code,
                        method,
                        path,
                        elapsed
                    );
                }

                Err(ref err) => {
                    println!(
                        "\x1B[34m[{}]\x1B[91m Error\x1B[97m {:6} {} ({:?}) => {:?}",
                        Utc::now().format("%Y-%m-%d %H:%M:%S"),
                        method,
                        path,
                        elapsed,
                        err
                    );
                }
            }
        }

        resp
    }
}

fn full<T>(chunk: T) -> BoxBody<Bytes, hyper::Error>
where
    T: Into<Bytes>,
{
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
