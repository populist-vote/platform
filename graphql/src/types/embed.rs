use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use chrono::{DateTime, Utc};
use db::{Embed, UserWithProfile};
use serde_json::Value as JSON;

use crate::context::ApiContext;

use super::{BillResult, Error, UserResult};

#[derive(SimpleObject, Clone, Debug)]
#[graphql(complex)]
pub struct EmbedResult {
    pub id: ID,
    pub organization_id: ID,
    pub name: String,
    pub description: Option<String>,
    pub populist_url: String,
    pub attributes: JSON,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_id: ID,
    pub updated_by_id: ID,
}

#[ComplexObject]
impl EmbedResult {
    async fn created_by(&self, ctx: &Context<'_>) -> Result<UserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            UserWithProfile,
            r#"
            SELECT u.id, u.username, u.email, first_name, last_name, profile_picture_url FROM user_profile up
            JOIN populist_user u ON up.user_id = u.id WHERE u.id = $1
        "#,
            uuid::Uuid::parse_str(&self.created_by_id.as_str()).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        match record {
            Some(user) => Ok(user.into()),
            None => Err(Error::UserNotFound.into()),
        }
    }

    async fn updated_by(&self, ctx: &Context<'_>) -> Result<UserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            UserWithProfile,
            r#"
            SELECT u.id, u.username, u.email, first_name, last_name, profile_picture_url FROM user_profile up
            JOIN populist_user u ON up.user_id = u.id WHERE u.id = $1
        "#,
            uuid::Uuid::parse_str(&self.updated_by_id.as_str()).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        match record {
            Some(user) => Ok(user.into()),
            None => Err(Error::UserNotFound.into()),
        }
    }

    async fn bill(&self, ctx: &Context<'_>) -> Result<Option<BillResult>> {
        let bill_id = self.attributes["billId"].as_str();
        if let Some(bill_id) = bill_id {
            let bill_id = uuid::Uuid::parse_str(bill_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Bill::find_by_id(&db_pool, bill_id).await?;

            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }
}

impl From<Embed> for EmbedResult {
    fn from(embed: Embed) -> Self {
        Self {
            id: embed.id.into(),
            organization_id: embed.organization_id.into(),
            name: embed.name,
            description: embed.description,
            populist_url: embed.populist_url,
            attributes: embed.attributes,
            created_at: embed.created_at,
            updated_at: embed.updated_at,
            created_by_id: embed.created_by.into(),
            updated_by_id: embed.updated_by.into(),
        }
    }
}
