use std::str::FromStr;

use async_graphql::{
    extensions::{
        apollo_persisted_queries::{ApolloPersistedQueries, LruCacheStorage},
        ApolloTracing,
    },
    http::{playground_source, GraphQLPlaygroundConfig},
    Request, Response,
};

use dotenv::dotenv;
use graphql::{context::ApiContext, new_schema, PopulistSchema};
use log::info;
use poem::{
    get, handler,
    http::HeaderMap,
    listener::TcpListener,
    middleware::{Compression, Cors},
    web::{Data, Html, Json},
    EndpointExt, IntoResponse, Route, Server,
};
use regex::Regex;
use serde_json::Value;
use server::Environment;

#[handler]
fn root() -> impl IntoResponse {
    Html(r#"<h1>Populist API Docs</h1>"#)
}

/// Simple server health check
// TODO: Make this a GraphQL subscription
#[handler]
fn ping() -> Json<Value> {
    Json(serde_json::json!({
        "ok": true
    }))
}

#[handler]
async fn graphql_handler(
    schema: Data<&PopulistSchema>,
    req: Json<Request>,
    headers: &HeaderMap,
) -> Json<Response> {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());

    Json(schema.execute(req.0.data(token)).await)
}

#[handler]
fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

pub fn cors(environment: Environment) -> Cors {
    let cors = Cors::new();

    fn allowed_staging_origins(origin: &str) -> bool {
        let staging_origins = vec![
            "https://populist-api-staging.herokuapp.com",
            "https://api.staging.populist.us",
            "https://staging.populist.us",
            "http://localhost:3030",
        ];
        let re = Regex::new(r"https://web-.*?-populist\.vercel\.app$").unwrap();
        re.is_match(origin) || staging_origins.contains(&origin)
    }

    match environment {
        Environment::Local => cors,
        Environment::Staging => cors.allow_origins_fn(allowed_staging_origins),
        Environment::Production => cors.allow_origins(vec![
            "https://populist-api-production.herokuapp.com",
            "https://api.populist.us",
            "https://populist.us",
            "https://www.populist.us",
            "https://web-five-kohl.vercel.app",
            "https://web-populist.vercel.app",
            "https://web-git-main-populist.vercel.app",
        ]),
        _ => cors,
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    pretty_env_logger::init();

    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let context = ApiContext::new(pool.connection.clone());

    let schema = new_schema()
        .data(context)
        .extension(ApolloTracing)
        .extension(ApolloPersistedQueries::new(LruCacheStorage::new(256)))
        .finish();

    let environment = Environment::from_str(&std::env::var("ENVIRONMENT").unwrap()).unwrap();
    let app = Route::new()
        .at("/", get(graphql_playground).post(graphql_handler))
        .data(schema)
        .with(cors(environment))
        .with(Compression::default());

    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    let address = format!("0.0.0.0:{}", port);

    info!("GraphQL Playground live at http://localhost:{}", &port);

    let listener = TcpListener::bind(&address);
    Server::new(listener).run(app).await
}
