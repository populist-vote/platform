use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_poem::GraphQL;
use dotenv::dotenv;
use graphql::new_schema;
use log::info;
use poem::{
    get, handler,
    listener::TcpListener,
    web::{Html, Json},
    IntoResponse, Route, Server,
};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;

// Simple server health check
#[handler]
fn ping() -> Json<Value> {
    Json(serde_json::json!({
        "ok": true
    }))
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

    let app = Route::new()
        .at("/status", get(ping))
        .at("/", get(graphql_playground).post(GraphQL::new(schema)));

    let environment = std::env::var("ENVIRONMENT").unwrap_or("production".to_string());
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let address = match environment.as_ref() {
        "local" => format!("127.0.0.1:{}", port),
        "production" => format!("80:{}", port),
        _ => format!("80:{}", port),
    };

    info!("GraphQL Playground live at {}", &address);

    let listener = TcpListener::bind(&address);
    let server = Server::new(listener).await?;
    server.run(app).await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}
