use async_graphql::extensions::ApolloTracing;
use axum::{extract::Extension, routing::get, Router, Server};
use dotenv::dotenv;
use graphql::{context::ApiContext, new_schema};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod cron;
pub use cron::init_job_schedule;
mod handlers;
pub use handlers::{graphql_handler, graphql_playground};

pub async fn app() -> Router {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Run cron jobs in seperate thread
    tokio::spawn(cron::init_job_schedule());

    // Embed migrations into binary
    let migrator = pool.connection.clone();
    sqlx::migrate!("../db/migrations")
        .run(&migrator)
        .await
        .unwrap();

    let context = ApiContext::new(pool.connection.clone());

    let schema = new_schema().data(context).extension(ApolloTracing).finish();

    axum::Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route_layer(CorsLayer::very_permissive())
        .layer(Extension(schema))
        .layer(CookieManagerLayer::new())
}

pub async fn run() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    info!("GraphQL Playground live at http://localhost:{}", &port);
    Server::bind(&format!("0.0.0.0:{}", port).parse().unwrap())
        .serve(app().await.into_make_service())
        .await
        .unwrap();
}
