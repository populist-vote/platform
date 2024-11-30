use async_graphql::{Data, Variables};
use sqlx::PgPool;
use std::{any::TypeId, net::SocketAddr};
use uuid::Uuid;

use crate::{
    context::{ApiContext, DataLoaders},
    new_schema, PopulistSchema, SessionData,
};

pub struct TestHarness {
    pub pool: PgPool,
}

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
        let context = ApiContext::new(self.pool.clone());

        context
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

    /// Executes a GraphQL query with a fresh context
    pub async fn execute_query<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        variables: Option<Variables>,
    ) -> anyhow::Result<T> {
        let context = self.create_context();

        let request = if let Some(vars) = variables {
            async_graphql::Request::new(query).variables(vars)
        } else {
            async_graphql::Request::new(query)
        };

        let schema = new_schema().data(context).finish();
        let response = schema.execute(request).await;

        if let Some(error) = response.errors.first() {
            return Err(anyhow::anyhow!("GraphQL error: {:?}", error));
        }

        Ok(serde_json::from_value(response.data.into_json()?)?)
    }
}
