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
    post,
    web::{Data, Html, Json},
    middleware::{Cors},
    IntoResponse, Route, Server, EndpointExt
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

    let app = Route::new()
        .at("/status", get(ping))
        .at("/playground", get(graphql_playground))
        .at("/", post(graphql_handler)).data(schema)
        .with( Cors::new()
        .allow_origin("localhost") 
        .allow_method(Method::POST));

    let port = std::env::var("PORT").unwrap_or("1234".to_string());
    let address = format!("0.0.0.0:{}", port);

    info!("GraphQL Playground live at {}/playground", &address);

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
