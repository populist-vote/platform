use std::str::FromStr;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Request, Response,
};
use dotenv::dotenv;
use graphql::{new_schema, PopulistSchema};
use log::info;
use poem::{
    get, handler,
    http::{HeaderMap, Method},
    listener::TcpListener,
    middleware::Cors,
    post,
    web::{Data, Html, Json},
    EndpointExt, IntoResponse, Route, Server,
};
use serde_json::Value;
use server::{Environment, Error};
use sqlx::postgres::PgPoolOptions;

// Simple server health check
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    pretty_env_logger::init();

    let db_url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&db_url)
        .await?;

    let schema = new_schema(pool).finish();

    let environment =
        Environment::from_str(&std::env::var("ENVIRONMENT").unwrap().to_string()).unwrap();

    let cors = Cors::new();
    let cors = match environment {
        Environment::Local => cors.allow_origin("http://localhost:1234"),
        Environment::Staging => cors
            .allow_origin("https://populist-api-staging.herokuapp.com")
            .allow_origin("http://localhost:3030"),
        Environment::Production => cors.allow_origin("https://populist-api-production.herokuapp.com/"),
        _ => Cors::new().allow_origin("https://populist.us"),
    };

    let app = Route::new()
        .at("/status", get(ping))
        .at("/playground", get(graphql_playground))
        .at("/", post(graphql_handler))
        .data(schema)
        .with(cors.allow_method(Method::POST).allow_method(Method::GET));

    let port = std::env::var("PORT").unwrap_or("1234".to_string());
    let address = format!("0.0.0.0:{}", port);

    info!("GraphQL Playground live at {}/playground", &address);

    let listener = TcpListener::bind(&address);
    let server = Server::new(listener).await?;
    server.run(app).await?;
    Ok(())
}
