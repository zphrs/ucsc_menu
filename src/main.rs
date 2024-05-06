// #![allow(dead_code)] // TODO: remove this line once done with crate
// #![allow(unused_imports)] // TODO: remove this line once done with crate
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod cache;
mod error;
mod fetch;
mod parse;
mod transpose;

use log;

use std::{
    env,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    body::Body,
    extract::{Request, State, WebSocketUpgrade},
    http::HeaderValue,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use futures::stream::{BoxStream, StreamExt as _};
use juniper::{
    graphql_object, graphql_subscription, Context, EmptyMutation, EmptySubscription, FieldError,
    RootNode,
};
use juniper_axum::{
    extract::JuniperRequest,
    graphiql, graphql, playground,
    response::JuniperResponse,
    subscriptions::{self, serve_graphql_transport_ws, serve_graphql_ws},
    ws,
};
use juniper_graphql_ws::ConnectionConfig;
use parse::Locations;
use tokio::{
    net::TcpListener,
    sync::OnceCell,
    time::{interval, sleep},
};
use tower_http::compression::CompressionLayer;

use crate::{cache::MultithreadedCache, fetch::make_client};

#[derive(Clone, Copy, Debug)]
pub struct Query;

static CACHE: OnceCell<MultithreadedCache<'static>> = OnceCell::const_new();
#[graphql_object]
impl Query {
    /// Adds two `a` and `b` numbers.
    async fn query(&self) -> Locations<'static> {
        let c = CACHE.get_or_init(|| async { MultithreadedCache::new().await.unwrap() });
        c.await.get().await.locations().to_owned()
    }
    #[graphql(ignore)]
    pub async fn refresh(self) {
        let c = CACHE.get_or_init(|| async { MultithreadedCache::new().await.unwrap() });
        let _ = c.await.refresh().await;
    }
}

impl Query {}

#[derive(Clone, Copy, Debug)]
pub struct Subscription;

type NumberStream = BoxStream<'static, Result<i32, FieldError>>;

type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn refresh<'a>() -> Response {
    let cache = CACHE
        .get_or_init(|| async { MultithreadedCache::new().await.unwrap() })
        .await;
    let _res = cache.refresh().await;
    Response::builder()
        .status(201)
        .body(Body::from("OK"))
        .unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    CACHE
        .get_or_init(|| async { MultithreadedCache::new().await.unwrap() })
        .await;
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str()).unwrap();
    let schema = Schema::new(Query, EmptyMutation::new(), EmptySubscription::new());
    let comression_layer: CompressionLayer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);
    pretty_env_logger::init_custom_env("ucsc_menu=info");

    let app = Router::new()
        .route(
            "/graphql",
            on(
                MethodFilter::GET.or(MethodFilter::POST),
                graphql::<Arc<Schema>>,
            ),
        )
        .route(
            "/subscriptions",
            get(ws::<Arc<Schema>>(ConnectionConfig::new(()))),
        )
        .route("/request-refresh", on(MethodFilter::PUT, refresh))
        .route("/graphiql", get(graphiql("/graphql", "/subscriptions")))
        .route("/playground", get(playground("/graphql", "/subscriptions")))
        .layer(Extension(Arc::new(schema)))
        .layer(comression_layer);
    tokio::spawn(async move {
        let client = make_client();
        log::info!("Forcing refresh");
        let _res = client
            .put(format!("http://{addr}/request-refresh"))
            .send()
            .await;
        log::info!("Forcing refresh done");
        sleep(Duration::from_secs(15 * 60)).await;
    });
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));
    log::info!("listening on http://{addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));
}
