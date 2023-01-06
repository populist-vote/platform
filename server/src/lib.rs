use async_graphql::extensions::ApolloTracing;
use axum::{
    body::Body,
    extract::Extension,
    headers::HeaderMap,
    http::Request,
    routing::{any, get},
    Router, Server,
};
use config::Environment;
use graphql::{context::ApiContext, new_schema};
use http::{request::Parts, HeaderValue};
use regex::Regex;
mod handlers;
use dotenv::dotenv;
pub use handlers::{graphql_handler, graphql_playground};
use std::str::FromStr;
use tower::util::ServiceExt;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
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

pub fn cors(environment: Environment) -> CorsLayer {
    let cors = CorsLayer::new().allow_credentials(true);

    fn allowed_staging_origins(origin: &HeaderValue, _request_parts: &Parts) -> bool {
        let staging_origins = vec![
            "https://populist-api-staging.herokuapp.com",
            "https://api.staging.populist.us",
            "https://staging.populist.us",
            "http://localhost:3030",
        ];
        let re = Regex::new(r"https://web-.*?-populist\.vercel\.app$").unwrap();
        re.is_match(origin.to_str().unwrap_or_default())
            || staging_origins.contains(&origin.to_str().unwrap_or_default())
    }

    let production_origins = [
        "http://localhost:3030".parse().unwrap(),
        "https://populist-api-production.herokuapp.com"
            .parse()
            .unwrap(),
        "https://api.populist.us".parse().unwrap(),
        "https://populist.us".parse().unwrap(),
        "https://www.populist.us".parse().unwrap(),
        "https://web-five-kohl.vercel.app".parse().unwrap(),
        "https://web-populist.vercel.app".parse().unwrap(),
        "https://web-git-main-populist.vercel.app".parse().unwrap(),
    ];

    match environment {
        Environment::Local => cors,
        Environment::Staging => cors.allow_origin(AllowOrigin::predicate(allowed_staging_origins)),
        Environment::Production => cors.allow_origin(production_origins),
        _ => cors,
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

    let environment = Environment::from_str(&std::env::var("ENVIRONMENT").unwrap()).unwrap();
    let context = ApiContext::new(pool.connection.clone());

    let schema = new_schema().data(context).extension(ApolloTracing).finish();

    // Use a permissive CORS policy to allow external requests
    let api_cors = CorsLayer::permissive();
    let web_cors = cors(environment);

    let api_router = axum::Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(api_cors);

    let web_router = axum::Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(web_cors);

    axum::Router::new()
        .route(
            "/",
            any(move |request: Request<Body>| async move {
                let request_type = determine_request_type(environment, request.headers());
                match request_type {
                    RequestType::Internal => web_router.oneshot(request).await,
                    RequestType::External => api_router.oneshot(request).await,
                }
            }),
        )
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
