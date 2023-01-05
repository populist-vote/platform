use pretty_assertions::{assert_eq, assert_ne};
use serde_json::{json, Value};
use std::net::{SocketAddr, TcpListener};

use server::app;

fn spawn_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:1234".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app().await.into_make_service())
            .await
            .unwrap();
    });
    addr
}

#[tokio::test]
async fn test_graphql_handler() {
    let addr = spawn_server();
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://{}/", addr))
        .json(&json!({
            "query": "query { health }",
        }))
        .send()
        .await
        .unwrap();

    let json = response.json::<Value>().await.unwrap();
    assert_eq!(json["data"], serde_json::json!({"health": true}));
}
