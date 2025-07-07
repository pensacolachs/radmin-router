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

#[derive(Clone, Debug)]
pub struct Router<Extra: Debug + Send + Sync> {
    ex: Arc<Extra>,
    root: Node<Extra>,
    route_not_found: RouteNotFoundHandler<Extra>,
    method_not_allowed: MethodNotAllowedHandler<Extra>,
}

impl<Extra: Debug + Send + Sync + 'static> Router<Extra> {
    pub fn new(ex: impl Into<Arc<Extra>>) -> Self {
        Self {
            ex: ex.into(),
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
                Box::pin(async move {
                    let allowed_methods = route
                        .allowed_methods()
                        .into_iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .header(header::ALLOW, allowed_methods)
                        .body(full("Method Not Allowed"))
                        .unwrap())
                })
            },
        }
    }

    pub fn route_not_found(&mut self, handler: RouteNotFoundHandler<Extra>) -> &mut Self {
        self.route_not_found = handler;
        self
    }

    pub fn method_not_allowed(&mut self, handler: MethodNotAllowedHandler<Extra>) -> &mut Self {
        self.method_not_allowed = handler;
        self
    }

    pub fn register(mut self, route: &Route<Extra>) -> Self {
        self.root.append(route.clone());
        self
    }

    pub fn register_many(&mut self, routes: impl IntoIterator<Item = Route<Extra>>) -> &mut Self {
        for route in routes {
            println!("Added route: {}", route.path);
            self.root.append(route);
        }

        self
    }

    pub fn match_route(&self, path: impl AsRef<str>) -> Option<(Route<Extra>, Vec<String>)> {
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
