use async_graphql::extensions::ApolloTracing;
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use dotenv::dotenv;
use graphql::{cache::Cache, context::ApiContext, new_schema};
use metrics::metrics_auth;
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;
mod cron;
pub mod jobs;
pub mod metrics;
mod postgres;
pub mod slack;
pub use cron::init_job_schedule;
pub use jobs::*;
mod handlers;
pub use handlers::{graphql_handler, graphql_playground};

pub async fn run() {
    dotenv().ok();
    CryptoProvider::install_default(default_provider())
        .expect("Failed to install default crypto provider");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    metrics::init_metrics();

    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    metrics::update_db_connections("main", pool);
    tokio::spawn(async move {
        let update_interval = Duration::from_secs(15);
        let pool = db::pool().await;
        loop {
            metrics::update_db_connections("main", pool);
            tokio::time::sleep(update_interval).await;
        }
    });

    // Run cron jobs in separate thread
    tokio::spawn(cron::init_job_schedule());

    // Postgres realtime listeners in separate thread
    let pool_for_listener = pool.clone(); // No need to clone the actual connection pool

    tokio::spawn(async move {
        if let Err(e) = postgres::listener(pool_for_listener.connection).await {
            eprintln!("Error in listener: {}", e);
        }
    });

    // Embed migrations into binary
    sqlx::migrate!("../db/migrations")
        .run(&pool.connection)
        .await
        .unwrap();

    let context = ApiContext::new(pool.clone().connection);

    let environment = config::Config::default().environment;
    let cache_duration = if environment != config::Environment::Production {
        Duration::from_secs(10)
    } else {
        Duration::from_secs(60 * 30)
    };

    let mut schema_builder = new_schema()
        .data(context)
        .data(Cache::<String, serde_json::Value>::new(cache_duration))
        .extension(metrics::PrometheusMetricsExtension);

    if environment != config::Environment::Production {
        schema_builder = schema_builder.extension(ApolloTracing);
    }

    let schema = schema_builder.finish();

    let rustls_config = RustlsConfig::from_pem_file(
        PathBuf::from("server/src/certs/fullchain.pem"),
        PathBuf::from("server/src/certs/localhost+2-key.pem"),
    )
    .await
    .unwrap();

    let app = axum::Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .nest(
            "/metrics",
            axum::Router::new()
                .route("/", get(metrics::metrics_handler))
                .layer(axum::middleware::from_fn(metrics_auth)),
        )
        .with_state(schema)
        .layer(axum::middleware::from_fn(metrics::track_metrics))
        .layer(CorsLayer::very_permissive())
        .layer(CookieManagerLayer::new());

    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    let https_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    info!("GraphQL Playground live at https://localhost:{}", &port);
    axum_server::bind_rustls(https_addr, rustls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
