use std::error::Error;
use std::net::{SocketAddr};
use std::sync::Arc;
use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full};
use http_body_util::combinators::BoxBody;
use hyper::{header, Response, StatusCode};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use radmin_router::path;
use radmin_router::route::Route;
use radmin_router::router::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bind_addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    let listener = TcpListener::bind(bind_addr).await?;

    let router = Router::new(())
        .register(
            Route::new(
                path!("/")
            )
                .get(|_, _| Box::pin(async {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .header(header::CONTENT_TYPE, "text/plain; charset=UTF-8")
                        .body(full("OK"))
                        .unwrap())
                }))
                .options(|_, _| Box::pin(async {
                    Ok(Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .header("Access-Control-Allow-Private-Network", "true")
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(full(""))
                        .unwrap())
                }))
        )
        .register(
            Route::new(
                path!("/[slug]")
            )
                .get(|_, ctx| Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(full(ctx.params[0].clone()))
                        .unwrap())
                }))
        )
        .register(
            Route::new(
                path!("/[slug]/literal")
            )
                .get(|_, ctx| Box::pin(async move {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(full(format!("{} + literal", ctx.params[0])))
                    .unwrap())
            }))
        )
        .register(
            Route::new(
                path!("/[slug]/literal/[slug2]")
            )
                .get(|_, ctx| Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(full(format!("slugs: {} and {}", ctx.params[0], ctx.params[1])))
                        .unwrap())
                }))
        );
    let router = Arc::new(router);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let router = Arc::clone(&router);

        tokio::task::spawn(async move {
            let router = Arc::clone(&router);
            let svc = service_fn(move |req| Router::route(Arc::clone(&router), req));

            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("Error: {:?}", err);
            }
        });
    }
}

fn full<I: Into<Bytes>>(body: I) -> BoxBody<Bytes, hyper::Error> {
    Full::new(body.into())
        .map_err(|e| match e {})
        .boxed()
}