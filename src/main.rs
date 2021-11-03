use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_poem::GraphQL;
use poem::{
    get, handler,
    listener::TcpListener,
    web::{Html, Json},
    IntoResponse, Route, Server,
};
use serde_json::Value;
use graphql::new_schema;
use dotenv::dotenv;
pub use db::{DatabasePool, DatabasePoolOptions};


// Simple server health check
#[handler]
fn ping() -> Json<Value> {
    Json(serde_json::json!({
        "ok": true
    }))
}

// #[handler]
// async fn graphql_handler(schema: PopulistSchema, req: Json<Request>) -> Json<Response> {
//     Json(schema.execute(req.0).await)
// }

#[handler]
fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("No DATABASE_URL");

    let db_pool = crate::DatabasePool::new_from_config(&db_url, &None).await;

    tracing_subscriber::fmt::init();
    let schema = new_schema().finish();

    let app = Route::new()
        .at("/status", get(ping))
        .at("/", get(graphql_playground).post(GraphQL::new(schema)));

    let listener = TcpListener::bind("127.0.0.1:3000");
    let server = Server::new(listener).await?;
    println!("Server is running on port 3000");
    server.run(app).await
}
