use async_graphql::extensions::ApolloTracing;
use axum::routing::get;
use dotenv::dotenv;
use graphql::{context::ApiContext, new_schema};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod cron;
pub mod jobs;
mod postgres;
pub use cron::init_job_schedule;
pub use jobs::*;
mod handlers;
pub use handlers::{graphql_handler, graphql_playground};

pub async fn run() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

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

    let schema = new_schema().data(context).extension(ApolloTracing).finish();

    let app = axum::Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .with_state(schema)
        .layer(CorsLayer::very_permissive())
        .layer(CookieManagerLayer::new());

    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    info!("GraphQL Playground live at http://localhost:{}", &port);
    let address = format!("0.0.0.0:{}", port);
    axum::serve(
        TcpListener::bind(address).await.unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
