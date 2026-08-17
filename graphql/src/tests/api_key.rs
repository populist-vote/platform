#[cfg(test)]
mod tests {
    use crate::{new_schema, tests::harness::TestHarness};
    use async_graphql::Variables;
    use auth::{AccessTokenClaims, AuthenticationKind};
    use jsonwebtoken::{Header, TokenData};
    use serde_json::json;

    #[tokio::test]
    async fn user_can_create_use_list_and_revoke_an_api_key() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;
        let user_id = harness.create_user("api-user@example.com", None).await?;
        sqlx::query("UPDATE populist_user SET confirmed_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&harness.pool)
            .await?;

        let created: serde_json::Value = harness
            .execute_query(
                r#"
                mutation CreateApiKey($name: String!) {
                  createApiKey(name: $name) {
                    key
                    apiKey { id name prefix createdAt lastUsedAt }
                  }
                }
                "#,
                Some(Variables::from_json(json!({ "name": "Local development" }))),
                Some(user_id),
                None,
            )
            .await?;

        let plaintext = created["createApiKey"]["key"]
            .as_str()
            .expect("new key is returned once");
        let key_id = created["createApiKey"]["apiKey"]["id"]
            .as_str()
            .expect("key has an ID");
        assert!(plaintext.starts_with(auth::API_KEY_PREFIX));
        assert!(plaintext.starts_with(
            created["createApiKey"]["apiKey"]["prefix"]
                .as_str()
                .expect("key has a display prefix")
        ));

        let authentication = auth::authenticate_api_key(&harness.pool, plaintext)
            .await?
            .expect("the new key authenticates");
        assert_eq!(authentication.claims.sub, user_id);

        let listed: serde_json::Value = harness
            .execute_query(
                "query ApiKeys { apiKeys { id name prefix } }",
                None,
                Some(user_id),
                None,
            )
            .await?;
        assert_eq!(listed["apiKeys"].as_array().unwrap().len(), 1);
        assert_eq!(listed["apiKeys"][0]["name"], "Local development");

        let revoked: serde_json::Value = harness
            .execute_query(
                "mutation RevokeApiKey($id: ID!) { revokeApiKey(id: $id) }",
                Some(Variables::from_json(json!({ "id": key_id }))),
                Some(user_id),
                None,
            )
            .await?;
        assert_eq!(revoked["revokeApiKey"], true);
        assert!(auth::authenticate_api_key(&harness.pool, plaintext)
            .await?
            .is_none());

        harness.cleanup().await?;
        Ok(())
    }

    #[tokio::test]
    async fn api_key_cannot_create_another_api_key() {
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            username: "api-user".to_string(),
            email: "api-user@example.com".to_string(),
            system_role: db::SystemRoleType::User,
            organizations: vec![],
            exp: usize::MAX,
        };
        let schema = new_schema()
            .data(Some(TokenData {
                header: Header::default(),
                claims,
            }))
            .data(Some(AuthenticationKind::ApiKey))
            .finish();

        let response = schema
            .execute(r#"mutation { createApiKey(name: "persistence") { key } }"#)
            .await;

        assert_eq!(response.errors.len(), 1);
        assert!(response.errors[0]
            .message
            .contains("interactive signed-in session"));
    }
}
