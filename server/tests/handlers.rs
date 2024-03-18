use pretty_assertions::assert_eq;
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

    let cookies = response.cookies();
    let mut has_session_id_cookie = false;
    let mut session_id_is_valid = false;

    for cookie in cookies {
        if cookie.name() == "session_id" {
            has_session_id_cookie = true;
            session_id_is_valid = uuid::Uuid::parse_str(cookie.value()).is_ok();
        }
    }

    assert!(has_session_id_cookie);
    assert!(session_id_is_valid);

    // Now you can check the cookies as needed

    let json = response.json::<Value>().await.unwrap();
    assert_eq!(json["data"], serde_json::json!({"health": true}));
}
