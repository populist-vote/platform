use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_poem::GraphQL;
use db::models::politician::CreatePoliticianInput;
use dotenv::dotenv;
use graphql::new_schema;
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
    // let db_url = "postgresql://wiley@localhost/populist-test";

    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&db_url)
        .await?;

    // let default_politician_input = CreatePoliticianInput {
    //     first_name: "test".to_string(),
    //     middle_name: None,
    //     last_name: "test_last".to_string(),
    //     nickname: None,
    //     preferred_name: None,
    //     ballot_name: None,
    //     description: None,
    //     thumbnail_image_url: None,
    //     home_state: "CO".to_string(),
    //     website_url: None,
    //     facebook_url: None,
    //     twitter_url: None,
    //     instagram_url: None,
    // };

    // let new_politician_input = CreatePoliticianInput {
    //     first_name: "Betsy".to_string(),
    //     // middle_name: Some("Ornate".to_string()),
    //     last_name: "Ross".to_string(),
    //     home_state: "NY".to_string(),
    //     ..default_politician_input
    // };

    // db::models::politician::Politician::create(&pool, &new_politician_input).await?;

    // db::models::politician::Politician::update(
    //     &pool,
    //     uuid::Uuid::parse_str("f56fedc2-970c-42cb-9fd9-08b146520b9a").unwrap(),
    //     &new_politician_input,
    // )
    // .await?;

    let schema = new_schema(pool).finish();

    let app = Route::new()
        .at("/status", get(ping))
        .at("/", get(graphql_playground).post(GraphQL::new(schema)));

    println!("Playground: http://localhost:3000");
    
    let listener = TcpListener::bind("127.0.0.1:3000");
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
