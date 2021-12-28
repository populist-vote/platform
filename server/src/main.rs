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
    http::{header, HeaderMap, Method},
    listener::TcpListener,
    middleware::{Cors, ForceHttps},
    web::{Data, Html, Json},
    EndpointExt, IntoResponse, Route, Server,
};
use serde_json::Value;
use server::{Environment, Error};
use sqlx::postgres::PgPoolOptions;

#[handler]
fn root() -> impl IntoResponse {
    Html(
        r#"
        <h1>Populist API Docs</h1>
    "#,
    )
}

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

    let environment = Environment::from_str(&std::env::var("ENVIRONMENT").unwrap()).unwrap();

    let cors = Cors::default()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![
            header::ACCEPT,
            header::ACCEPT_ENCODING,
            header::ACCEPT_LANGUAGE,
            header::AUTHORIZATION,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            header::ACCESS_CONTROL_ALLOW_METHODS,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            header::CONNECTION,
            header::CONTENT_LENGTH,
            header::CONTENT_TYPE,
            header::HOST,
            header::ORIGIN,
            header::REFERER,
            header::UPGRADE,
            header::USER_AGENT,
        ])
        .expose_headers(vec![
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::ACCESS_CONTROL_ALLOW_METHODS,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ]);

    match environment {
        Environment::Local => cors.allow_origin("http://localhost:1234"),
        Environment::Staging => cors.allow_origins(vec![
            "https://populist-api-staging.herokuapp.com",
            "http://localhost:3030",
        ]),
        Environment::Production => {
            cors.allow_origin("https://populist-api-production.herokuapp.com/")
        }
        _ => Cors::new().allow_origin("https://populist.us"),
    };

    let app = Route::new()
        .at("/", get(graphql_playground).post(graphql_handler))
        .data(schema)
        .with(Cors::default())
        .with(ForceHttps::default());

    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    let address = format!("0.0.0.0:{}", port);

    info!("GraphQL Playground live at {}/playground", &address);

    let listener = TcpListener::bind(&address);
    let server = Server::new(listener);
    server.run(app).await?;
    Ok(())
}
