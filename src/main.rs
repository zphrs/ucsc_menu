#![allow(dead_code)] // TODO: remove this line once done with crate
#![allow(unused_imports)] // TODO: remove this line once done with crate
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod cache;
mod error;
mod fetch;
mod parse;
mod transpose;

use std::{
    convert::Infallible, env, error::Error, net::SocketAddr, str::FromStr, sync::Arc,
    time::Duration,
};

use axum::http::HeaderValue;
use hyper::{
    server::conn::http1, server::conn::http2, service::service_fn, Method, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use juniper_hyper::{graphiql, graphql, playground};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tower_http::compression::CompressionLayer;
use tracing::trace;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env::set_var("RUST_LOG", "ucsc_menu=info");
    pretty_env_logger::init();

    let db = Arc::new(cache::MultithreadedCache::new().await.unwrap());
    let addr = SocketAddr::from_str(&format!(
        "{}:{}",
        env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("PORT").unwrap_or_else(|_| "3000".to_string())
    ))
    .unwrap();
    let listener = TcpListener::bind(addr).await?;
    let comression_layer: CompressionLayer = CompressionLayer::new().gzip(true);
    log::info!("Listening on http://{addr}");
    tokio::spawn(async move {
        loop {
            log::info!("Forcing a cache refresh");
            // let n = Instant::now();
            // db1.refresh().await.unwrap();
            // log::info!("Finished refreshing cache in {:?}", n.elapsed());
            let client = fetch::make_client();
            let url = Url::from_str(&format!("https://graphql.ucsc.menu/request-refresh")).unwrap();
            let _ = client.patch(url).send().await;
            log::info!("Forced cache refresh done");
            sleep(Duration::from_secs(15 * 60)).await;
        }
    });
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let db = db.clone();
        // let root_node = root_node.clone();
        tokio_scoped::scope(|scope| {
            let s = scope;
            let db = db.clone();
            s.spawn(async move {
                // let root_node = root_node.clone();

                if let Err(e) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(|req| async {
                            let root_node = Arc::new(db.get_root_node().await);
                            Ok::<_, Infallible>(match (req.method(), req.uri().path()) {
                                (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                                    let mut res = graphql(root_node, Arc::new(()), req).await;
                                    res.headers_mut().append(
                                        "Access-Control-Allow-Origin",
                                        HeaderValue::from_static("*"),
                                    );
                                    res
                                }
                                (&Method::GET, "/graphiql") => graphiql("/graphql", None).await,
                                (&Method::GET, "/playground") => playground("/graphql", None).await,
                                (&Method::PATCH, "/request-refresh") => {
                                    log::info!("Refreshing cache");
                                    let n = Instant::now();
                                    db.refresh().await.unwrap();
                                    log::info!("Finished refreshing cache in {:?}", n.elapsed());
                                    let mut resp = Response::new(String::new());
                                    *resp.status_mut() = StatusCode::CREATED;
                                    resp.body_mut().push_str("OK");
                                    resp
                                }
                                _ => {
                                    let mut resp = Response::new(String::new());
                                    *resp.status_mut() = StatusCode::NOT_FOUND;
                                    resp
                                }
                            })
                        }),
                    )
                    .await
                {
                    log::error!("Error serving connection: {e}");
                }
            });
        });
    }
}
