use std::net::SocketAddr;

use async_graphql::Variables;
use auth::{create_random_token, create_temporary_username, AccessTokenClaims};
use db::{AddressInput, CreateUserWithProfileInput};
use jsonwebtoken::{Header, TokenData};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{context::ApiContext, new_schema, SessionData};

#[derive(Clone)]
#[allow(dead_code)]
pub struct TestHarness {
    pub pool: PgPool,
}

#[allow(dead_code)]
impl TestHarness {
    /// Creates a new test harness with a fresh database
    pub async fn new() -> anyhow::Result<Self> {
        // Set up fresh test database
        let admin_pool = PgPool::connect("postgres://localhost/postgres").await?;
        sqlx::query("DROP DATABASE IF EXISTS populist_test WITH (FORCE)")
            .execute(&admin_pool)
            .await?;
        sqlx::query("CREATE DATABASE populist_test")
            .execute(&admin_pool)
            .await?;

        let pool = PgPool::connect("postgres://localhost/populist_test").await?;

        // Run migrations
        sqlx::migrate!("../db/migrations").run(&pool).await?;

        let harness = Self { pool };
        harness.clear_tables().await?;

        Ok(harness)
    }

    /// Creates a new user with the given permissions and returns their ID
    pub async fn create_user(
        &self,
        email: &str,
        address: Option<AddressInput>,
    ) -> anyhow::Result<Uuid> {
        let confirmation_token = create_random_token().unwrap();
        let temp_username = create_temporary_username(email.to_string());
        let input = CreateUserWithProfileInput {
            address,
            email: email.to_string(),
            username: temp_username,
            password: "password".to_string(),
            confirmation_token,
        };

        let user = db::User::create_with_profile(&self.pool, &input).await?;

        Ok(user.id)
    }

    /// Creates a test organization and returns its ID
    pub async fn create_organization(&self, name: &str) -> anyhow::Result<Uuid> {
        let organization_id = Uuid::new_v4();
        let slug = name.to_lowercase().replace(' ', "-");

        sqlx::query!(
            "INSERT INTO organization (id, name, slug) VALUES ($1, $2, $3)",
            organization_id,
            name,
            slug
        )
        .execute(&self.pool)
        .await?;

        Ok(organization_id)
    }

    /// Creates a fresh context for GraphQL operations
    fn create_context(&self) -> ApiContext {
        ApiContext::new(self.pool.clone())
    }

    /// Clears all tables in the database
    pub async fn clear_tables(&self) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            TRUNCATE TABLE 
                conversation, 
                statement, 
                statement_view, 
                statement_vote,
                organization,
                politician,
                office,
                race,
                issue_tag,
                populist_user,
                user_profile
            CASCADE
            "#
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn execute_query<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        variables: Option<Variables>,
        user_id: Option<Uuid>,
        session_id: Option<Uuid>,
    ) -> anyhow::Result<T> {
        let context = self.create_context();

        let request = if let Some(vars) = variables {
            async_graphql::Request::new(query).variables(vars)
        } else {
            async_graphql::Request::new(query)
        };

        let mut schema = new_schema().data(context);

        if let Some(uid) = user_id {
            let claims = AccessTokenClaims {
                sub: uid,
                username: "test_user".to_string(),
                email: "test@example.com".to_string(),
                system_role: db::SystemRoleType::User,
                organizations: vec![],
                exp: usize::MAX,
            };
            schema = schema.data(Some(TokenData {
                header: Header::default(),
                claims,
            }));
        }

        if let Some(sid) = session_id {
            let test_socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
            schema = schema.data(SessionData {
                session_id: crate::SessionID(sid.to_string()),
                ip: test_socket_addr,
            });
        }

        let schema = schema.finish();
        let response = schema.execute(request).await;

        if let Some(error) = response.errors.first() {
            return Err(anyhow::anyhow!("GraphQL error: {:?}", error));
        }

        Ok(serde_json::from_value(response.data.into_json()?)?)
    }
}
