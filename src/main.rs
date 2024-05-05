#![allow(dead_code)] // TODO: remove this line once done with crate
#![allow(unused_imports)] // TODO: remove this line once done with crate
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod cache;
mod error;
mod fetch;
mod parse;
mod transpose;

use std::{convert::Infallible, env, error::Error, net::SocketAddr, sync::Arc, time::Duration};

use hyper::{server::conn::http1, service::service_fn, Method, Response, StatusCode};
use hyper_util::rt::TokioIo;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use juniper_hyper::{graphiql, graphql, playground};
use tokio::{net::TcpListener, time::sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    // let root_node = Arc::new(db.get_root_node().await);

    let db = Arc::new(cache::MultithreadedCache::new().await.unwrap());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on http://{addr}");
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let db = db.clone();
        // let root_node = root_node.clone();
        tokio_scoped::scope(|scope| {
            let mut s = scope;
            let db1 = db.clone();
            s = s.spawn(async move {
                loop {
                    sleep(Duration::from_secs(15 * 60)).await;
                    db1.refresh().await.unwrap();
                }
            });
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
