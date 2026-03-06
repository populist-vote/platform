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

        Ok(())
    }
}
