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

use hyper::{server::conn::http1, service::service_fn, Method, Response, StatusCode};
use hyper_util::rt::TokioIo;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use juniper_hyper::{graphiql, graphql, playground};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::trace;

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
    log::info!("Listening on http://{addr}");
    let db1 = db.clone();
    tokio::spawn(async move {
        loop {
            log::info!("Refreshing cache");
            let n = Instant::now();
            db1.refresh().await.unwrap();
            log::info!("Finished refreshing cache in {:?}", n.elapsed());
            sleep(Duration::from_secs(1 * 60)).await;
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
                                    graphql(root_node, Arc::new(()), req).await
                                }
                                (&Method::GET, "/graphiql") => graphiql("/graphql", None).await,
                                (&Method::GET, "/playground") => playground("/graphql", None).await,
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
