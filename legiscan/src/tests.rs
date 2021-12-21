#[cfg(test)]
mod tests {
    use crate::LegiscanProxy;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_session_list() {
        let proxy = LegiscanProxy::new().unwrap();
        let session_list = proxy.get_session_list("CO").await.unwrap();
        let json = serde_json::to_value(session_list).unwrap();
        assert_eq!(json, json!(null));
    }
}
