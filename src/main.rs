// #![allow(dead_code)] // TODO: remove this line once done with crate
// #![allow(unused_imports)] // TODO: remove this line once done with crate
#![deny(unused_crate_dependencies)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use axum_server as _; // to use rustls over openssl bc alpine linux

mod cache;
mod error;
mod fetch;
mod parse;
mod transpose;

use std::{
    env,
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    http::Method,
    response::Response,
    routing::{get, on, MethodFilter},
    Extension, Router,
};

use crate::{cache::Multithreaded, fetch::make_client};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, RootNode};
use juniper_axum::{graphiql, graphql, playground, ws};
use juniper_graphql_ws::ConnectionConfig;
use parse::Locations;
use tokio::{net::TcpListener, sync::OnceCell, time::sleep};
use tower_http::cors::CorsLayer;
use tower_http::{compression::CompressionLayer, cors::Any};

#[derive(Clone, Copy, Debug)]
pub struct Query;

static CACHE: OnceCell<Multithreaded<'static>> = OnceCell::const_new();
#[graphql_object]
impl Query {
    /// Adds two `a` and `b` numbers.
    async fn query(&self) -> Locations<'static> {
        let c = CACHE.get_or_init(|| async { Multithreaded::new().await.unwrap() });
        c.await.get().await.locations().to_owned()
    }
    #[graphql(ignore)]
    pub async fn refresh(self) {
        let c = CACHE.get_or_init(|| async { Multithreaded::new().await.unwrap() });
        let _ = c.await.refresh().await;
    }
}

impl Query {}

#[derive(Clone, Copy, Debug)]
pub struct Subscription;

type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn refresh<'a>() -> Response {
    let cache = CACHE
        .get_or_init(|| async { Multithreaded::new().await.unwrap() })
        .await;
    let _res = cache.refresh().await;
    let c = cache.get().await;
    Response::builder()
        .status(201)
        .body(Body::from(format!(
            "Last refresh: {}\nNext refresh: {}",
            c.get_time_since_refresh(),
            c.get_time_until_refresh(),
        )))
        .unwrap()
}

#[cfg(not(feature = "dump-schema"))]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    CACHE
        .get_or_init(|| async { Multithreaded::new().await.unwrap() })
        .await;
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from_str(format!("{host}:{port}").as_str()).unwrap();
    let schema = Schema::new(Query, EmptyMutation::new(), EmptySubscription::new());
    let comression_layer: CompressionLayer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);
    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST]) // intentionally excludes request-refresh/PUT
        .allow_origin(Any);
    pretty_env_logger::init();

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
        .layer(cors_layer)
        .layer(Extension(Arc::new(schema)))
        .layer(comression_layer);
    tokio::spawn(async move {
        let client = make_client();
        log::info!("Forcing refresh");
        let start = Instant::now();
        let _res = client
            .put(format!("http://{addr}/request-refresh"))
            .send()
            .await;
        log::info!("Forcing refresh done, took {:?}", start.elapsed());
        sleep(Duration::from_secs(
            cache::REFRESH_INTERVAL
                .num_seconds()
                .try_into()
                .expect("refresh interval to be positive"),
        ))
        .await;
    });
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));
    log::info!("listening on http://{addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));
}

#[cfg(feature = "dump-schema")]
fn main() {
    let schema = Schema::new(Query, EmptyMutation::new(), EmptySubscription::new());
    std::fs::write("ucsc_menu.graphql", schema.as_sdl().as_bytes())
        .expect("error writing schema to file");
}
