use async_graphql::extensions::ApolloTracing;
use config::Environment;
use graphql::{context::ApiContext, new_schema};
use poem::{http::HeaderMap, middleware::Cors, Route};
use regex::Regex;
mod handlers;
use async_graphql_poem::GraphQLSubscription;
use dotenv::dotenv;
pub use handlers::{graphql_handler, graphql_playground};
use poem::{
    get,
    listener::TcpListener,
    middleware::{Compression, CookieJarManager},
    EndpointExt, Server,
};
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

pub async fn run() -> std::io::Result<()> {
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

    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    let schema = new_schema().data(context).extension(ApolloTracing).finish();

    // Use a permissive CORS policy to allow external requests
    let cors = Cors::new().allow_credentials(true);

    let app = Route::new()
        .at("/", get(graphql_playground).post(graphql_handler))
        .at("/ws", get(GraphQLSubscription::new(schema.clone())))
        .data(schema)
        .with(cors)
        .with(Compression::default())
        .with(CookieJarManager::default());

    let address = format!("0.0.0.0:{}", port);
    info!("GraphQL Playground live at http://localhost:{}", &port);
    let listener = TcpListener::bind(&address);

    Server::new(listener).run(app).await
}
