// #![allow(dead_code)] // TODO: remove this line once done with crate
// #![allow(unused_imports)] // TODO: remove this line once done with crate
#![deny(unused_crate_dependencies)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use axum_server as _; // to use rustls over openssl bc alpine linux

mod cache;
mod config;
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
    extract::State,
    http::Method,
    response::Response,
    routing::{get, on, MethodFilter},
    Extension, Router,
};

use crate::{
    cache::{Multithreaded, Store},
    fetch::make_client,
};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, RootNode};
use juniper_axum::{graphiql, graphql, playground, ws};
use juniper_graphql_ws::ConnectionConfig;
use parse::Locations;
use tokio::{net::TcpListener, time::sleep};
use tower_http::cors::CorsLayer;
use tower_http::{compression::CompressionLayer, cors::Any};

pub use error::Result;

#[derive(Clone, Debug)]
pub struct Query(Arc<Multithreaded>);

#[graphql_object]
impl Query {
    /// Adds two `a` and `b` numbers.
    async fn query(&self) -> Locations {
        self.0.get().await.locations().to_owned()
    }
    #[graphql(ignore)]
    pub async fn refresh(self) {
        if let Err(e) = self.0.refresh().await {
            tracing::warn!("Error while refreshing cache: {e:?}");
        }
    }
}

impl Query {}

#[derive(Clone, Copy, Debug)]
pub struct Subscription;

type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn refresh(State(cache): State<Arc<Multithreaded>>) -> Response {
    let _res = cache.refresh().await;
    let c = cache.get().await;
    Response::builder()
        .status(201)
        .body(Body::from(format!(
            "Last refresh: {}\nNext refresh: {}",
            c.time_since_refresh(),
            c.time_until_refresh(),
        )))
        .unwrap()
}

#[cfg(not(feature = "dump-schema"))]
#[tokio::main(flavor = "current_thread")]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let store = match env::var("CACHE").as_deref() {
        Ok(":firestore:") => Store::cloud().await?,
        Ok(":memory:") => Store::AdHoc,
        Ok(p) => Store::local(p).await?,
        Err(_) => {
            log::warn!("env var CACHE not set, using ad-hoc memory cache.");
            Store::AdHoc
        }
    };
    println!("{store:?}");
    let cache = Arc::new(Multithreaded::new(store).await?);
    println!("{cache:?}");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from_str(format!("{host}:{port}").as_str()).unwrap();
    let schema = Schema::new(
        Query(Arc::clone(&cache)),
        EmptyMutation::new(),
        EmptySubscription::new(),
    );
    let comression_layer: CompressionLayer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);
    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST]) // intentionally excludes request-refresh/PUT
        .allow_origin(Any);

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
        .route("/graphiql", get(graphiql("/graphql", "/subscriptions")))
        .route("/playground", get(playground("/graphql", "/subscriptions")))
        .route("/request-refresh", on(MethodFilter::PUT, refresh))
        .with_state(Arc::clone(&cache))
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
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(feature = "dump-schema")]
fn main() {
    let schema = Schema::new(Query, EmptyMutation::new(), EmptySubscription::new());
    std::fs::write("ucsc_menu.graphql", schema.as_sdl().as_bytes())
        .expect("error writing schema to file");
}
