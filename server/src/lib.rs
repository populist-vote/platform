use async_graphql::extensions::ApolloTracing;
use axum::{extract::Extension, headers::HeaderMap, routing::get, Router, Server};
use config::Environment;
use graphql::{context::ApiContext, new_schema};
use regex::Regex;
mod handlers;
use dotenv::dotenv;
pub use handlers::{graphql_handler, graphql_playground};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn is_internal_staging_origin(origin: &str) -> bool {
    let staging_origins = vec![
        "https://populist-api-staging.herokuapp.com",
        "https://api.staging.populist.us",
        "https://staging.populist.us",
        "http://localhost:1234",
        "http://localhost:3030",
    ];
    let re = Regex::new(r"https://web-.*?-populist\.vercel\.app$").unwrap();
    re.is_match(origin) || staging_origins.contains(&origin)
}

fn is_internal_production_origin(origin: &str) -> bool {
    let production_origins = vec![
        "https://populist-api-production.herokuapp.com",
        "https://api.populist.us",
        "https://populist.us",
        "https://www.populist.us",
        "https://web-five-kohl.vercel.app",
        "https://web-populist.vercel.app",
        "https://web-git-main-populist.vercel.app",
    ];

    production_origins.contains(&origin)
}

#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    Internal,
    External,
}

pub fn determine_request_type(environment: Environment, headers: &HeaderMap) -> RequestType {
    let origin = headers
        .get("origin")
        .and_then(|header| header.to_str().ok());

    match environment {
        Environment::Staging => {
            if let Some(origin) = origin {
                if is_internal_staging_origin(origin) {
                    RequestType::Internal
                } else {
                    RequestType::External
                }
            } else {
                RequestType::External
            }
        }
        Environment::Production => {
            if let Some(origin) = origin {
                if is_internal_production_origin(origin) {
                    RequestType::Internal
                } else {
                    RequestType::External
                }
            } else {
                RequestType::External
            }
        }
        _ => RequestType::Internal,
    }
}

pub async fn app() -> Router {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Embed migrations into binary
    let migrator = pool.connection.clone();
    sqlx::migrate!("../db/migrations")
        .run(&migrator)
        .await
        .unwrap();

    let context = ApiContext::new(pool.connection.clone());

    let schema = new_schema().data(context).extension(ApolloTracing).finish();

    // Use a permissive CORS policy to allow external requests
    let cors = CorsLayer::permissive();

    axum::Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors)
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
