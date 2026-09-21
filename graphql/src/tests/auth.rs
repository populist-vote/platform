#[cfg(test)]
mod tests {
    use crate::tests::harness::TestHarness;

    #[tokio::test]
    async fn invite_existing_user_should_add_org_membership_after_login() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;
        let organization_id = harness.create_organization("Invite Flow Org").await?;

        let inviter_id = harness.create_user("inviter@example.com", None).await?;
        let invitee_id = harness.create_user("invitee@example.com", None).await?;

        // Seed inviter as org owner so they can run inviteUser.
        sqlx::query!(
            r#"
            INSERT INTO organization_users (organization_id, user_id, role)
            VALUES ($1, $2, 'owner')
            "#,
            organization_id,
            inviter_id
        )
        .execute(&harness.pool)
        .await?;

        let invite_mutation = r#"
            mutation InviteUser($input: InviteUserInput!) {
              inviteUser(input: $input)
            }
        "#;

        let invite_variables = serde_json::json!({
            "input": {
                // Use different casing to mirror real-world invite entry.
                "email": "Invitee@Example.com",
                "organizationId": organization_id.to_string()
            }
        });

        let _invite_response: serde_json::Value = harness
            .execute_query(
                invite_mutation,
                Some(async_graphql::Variables::from_json(invite_variables)),
                Some(inviter_id),
                None,
            )
            .await?;

        let login_mutation = r#"
            mutation LogIn($emailOrUsername: String!, $password: String!) {
              login(input: { emailOrUsername: $emailOrUsername, password: $password }) {
                userId
              }
            }
        "#;

        let login_variables = serde_json::json!({
            "emailOrUsername": "invitee@example.com",
            "password": "password"
        });

        let _login_response: serde_json::Value = harness
            .execute_query(
                login_mutation,
                Some(async_graphql::Variables::from_json(login_variables)),
                None,
                None,
            )
            .await?;

        let membership = sqlx::query!(
            r#"
            SELECT 1 AS exists
            FROM organization_users
            WHERE organization_id = $1 AND user_id = $2
            "#,
            organization_id,
            invitee_id
        )
        .fetch_optional(&harness.pool)
        .await?;

        assert!(
            membership.is_some(),
            "invited existing user should be added to organization membership after login"
        );

        harness.cleanup().await?;
        Ok(())
    }

    async fn seed_invite(
        harness: &TestHarness,
        email: &str,
        organization_id: uuid::Uuid,
    ) -> anyhow::Result<uuid::Uuid> {
        let inviter = harness.create_user("inviter@example.com", None).await?;
        Ok(sqlx::query_scalar(
            "INSERT INTO invite_token (email, organization_id, role, invited_by, expires_at)
             VALUES ($1, $2, 'admin', $3, now() AT TIME ZONE 'utc' + INTERVAL '7 days')
             RETURNING token",
        )
        .bind(email)
        .bind(organization_id)
        .bind(inviter)
        .fetch_one(&harness.pool)
        .await?)
    }

    #[tokio::test]
    async fn invited_registration_sets_both_cookies_and_org_membership() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;
        let organization_id = harness
            .create_organization("Registration Invite Org")
            .await?;
        let email = "staging.email.test+invited@example.com";
        let token = seed_invite(&harness, email, organization_id).await?;
        let schema = crate::new_schema()
            .data(crate::context::ApiContext::new(harness.pool.clone()))
            .data(None::<jsonwebtoken::TokenData<auth::AccessTokenClaims>>)
            .finish();
        let response = schema.execute(async_graphql::Request::new(
            r#"mutation Register($input: BeginUserRegistrationInput!) {
                beginUserRegistration(input: $input) { userId }
            }"#,
        ).variables(async_graphql::Variables::from_json(serde_json::json!({
            "input": { "email": email, "password": "VerySecurePassword", "inviteToken": token.to_string() }
        })))).await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        let cookies: Vec<_> = response
            .http_headers
            .get_all("set-cookie")
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect();
        assert_eq!(
            cookies.len(),
            2,
            "registration must retain both auth cookies"
        );
        let access = cookies
            .iter()
            .find(|cookie| cookie.starts_with("access_token="))
            .expect("access cookie")
            .split(';')
            .next()
            .unwrap()
            .strip_prefix("access_token=")
            .unwrap();
        let claims = auth::validate_access_token(access)?.claims;
        assert!(claims
            .organizations
            .iter()
            .any(|role| role.organization_id == organization_id
                && role.role == db::OrganizationRoleType::Admin));
        assert!(cookies
            .iter()
            .any(|cookie| cookie.starts_with("refresh_token=")));
        let accepted: bool =
            sqlx::query_scalar("SELECT accepted_at IS NOT NULL FROM invite_token WHERE token = $1")
                .bind(token)
                .fetch_one(&harness.pool)
                .await?;
        assert!(accepted);
        harness.cleanup().await?;
        Ok(())
    }

    #[tokio::test]
    async fn pending_invite_for_existing_user_requires_correct_password() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;
        let organization_id = harness.create_organization("Existing Invite Org").await?;
        let email = "existing@example.com";
        let user_id = harness.create_user(email, None).await?;
        let token = seed_invite(&harness, email, organization_id).await?;
        let mutation = r#"mutation Login($input: LoginInput!) {
            login(input: $input) { userId }
        }"#;
        for (password, succeeds) in [("incorrect", false), ("password", true)] {
            let result: anyhow::Result<serde_json::Value> = harness.execute_query(
                mutation,
                Some(async_graphql::Variables::from_json(serde_json::json!({
                    "input": { "emailOrUsername": email, "password": password, "inviteToken": token.to_string() }
                }))),
                None,
                None,
            ).await;
            assert_eq!(result.is_ok(), succeeds, "{result:?}");
            let accepted: bool = sqlx::query_scalar(
                "SELECT accepted_at IS NOT NULL FROM invite_token WHERE token = $1",
            )
            .bind(token)
            .fetch_one(&harness.pool)
            .await?;
            assert_eq!(accepted, succeeds);
            let roles = db::User::organization_roles(&harness.pool, user_id).await?;
            assert_eq!(
                roles
                    .iter()
                    .any(|role| role.organization_id == organization_id),
                succeeds
            );
        }
        harness.cleanup().await?;
        Ok(())
    }

    #[tokio::test]
    async fn invalid_invites_roll_back_registration_and_remain_unaccepted() -> anyhow::Result<()> {
        let harness = TestHarness::new().await?;
        let organization_id = harness.create_organization("Invalid Invite Org").await?;
        let invited_email = "staging.email.test+valid@example.com";
        let token = seed_invite(&harness, invited_email, organization_id).await?;
        let schema = crate::new_schema()
            .data(crate::context::ApiContext::new(harness.pool.clone()))
            .data(None::<jsonwebtoken::TokenData<auth::AccessTokenClaims>>)
            .finish();
        let cases = [
            ("malformed".to_string(), invited_email),
            (uuid::Uuid::new_v4().to_string(), invited_email),
            (token.to_string(), "staging.email.test+wrong@example.com"),
            (token.to_string(), invited_email),
        ];
        for (index, (invite, email)) in cases.iter().enumerate() {
            if index == 3 {
                sqlx::query("UPDATE invite_token SET expires_at = now() - INTERVAL '1 day' WHERE token = $1")
                    .bind(token).execute(&harness.pool).await?;
            }
            let response = schema.execute(async_graphql::Request::new(
                "mutation Register($input: BeginUserRegistrationInput!) { beginUserRegistration(input: $input) { userId } }",
            ).variables(async_graphql::Variables::from_json(serde_json::json!({
                "input": {"email": email, "password": "VerySecurePassword", "inviteToken": invite}
            })))).await;
            assert_eq!(
                response.errors.len(),
                1,
                "case {index}: {:?}",
                response.errors
            );
            assert!(response.errors[0]
                .message
                .contains("invitation is invalid or expired"));
            assert!(response.http_headers.get("set-cookie").is_none());
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM populist_user WHERE email = $1")
                    .bind(email)
                    .fetch_one(&harness.pool)
                    .await?;
            assert_eq!(
                count, 0,
                "failed registration must not leave an account behind"
            );
            let accepted: bool = sqlx::query_scalar(
                "SELECT accepted_at IS NOT NULL FROM invite_token WHERE token = $1",
            )
            .bind(token)
            .fetch_one(&harness.pool)
            .await?;
            assert!(!accepted);
        }
        harness.cleanup().await?;
        Ok(())
    }
}
