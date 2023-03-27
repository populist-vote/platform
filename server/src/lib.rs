use async_graphql::extensions::ApolloTracing;
use axum::{extract::Extension, routing::get, Router, Server};
use config::Environment;
use graphql::{context::ApiContext, new_schema};
use http::{request::Parts, HeaderValue};
use regex::Regex;
mod cron;
mod handlers;
pub use cron::init_job_schedule;
use dotenv::dotenv;
pub use handlers::{external_graphql_handler, graphql_playground, internal_graphql_handler};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

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
            let origin = origin.to_str().unwrap();
            re.is_match(origin) || staging_origins.contains(&origin)
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
            Environment::Staging => {
                cors.allow_origin(AllowOrigin::predicate(allowed_staging_origins))
            }
            Environment::Production => cors.allow_origin(production_origins),
            _ => cors,
        }
    }

    // Use a permissive CORS policy to allow external requests for /graphql endpoint
    let permissive_cors = CorsLayer::very_permissive();

    let environment = config::Config::default().environment;

    axum::Router::new()
        .route("/", get(graphql_playground).post(internal_graphql_handler))
        .route_layer(cors(environment))
        .route(
            "/graphql",
            get(graphql_playground).post(external_graphql_handler),
        )
        .route_layer(permissive_cors)
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
