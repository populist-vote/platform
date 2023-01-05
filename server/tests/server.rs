use poem::{test::TestClient, Route};
use server::{graphql_handler, graphql_playground};

#[tokio::test]
async fn test_graphql_playground() {
    let app = Route::new().at("/", graphql_playground);
    let client = TestClient::new(app);
    let resp = client.get("/").send().await;
    resp.assert_status_is_ok();
}
