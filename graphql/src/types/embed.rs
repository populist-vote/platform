use crate::is_admin;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{DateTime, Embed, EmbedType, UserWithProfile};
use serde_json::Value as JSON;
use tracing::log::warn;

use crate::{context::ApiContext, guard::StaffOnly};

use super::{BillResult, Error, PoliticianResult, PollResult, QuestionResult, UserResult};

#[derive(SimpleObject, Clone, Debug)]
#[graphql(complex)]
pub struct EmbedResult {
    pub id: ID,
    pub organization_id: ID,
    pub name: String,
    pub description: Option<String>,
    pub populist_url: String,
    pub embed_type: EmbedType,
    pub attributes: JSON,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub created_by_id: ID,
    pub updated_by_id: ID,
}

#[derive(SimpleObject)]
#[graphql(guard = "StaffOnly", visible = "is_admin")]
pub struct EmbedOriginResult {
    pub url: String,
    pub last_ping_at: DateTime,
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

    async fn origins(&self, ctx: &Context<'_>) -> Result<Vec<EmbedOriginResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            EmbedOriginResult,
            r#"
            SELECT url, last_ping_at FROM embed_origin WHERE embed_id = $1
        "#,
            uuid::Uuid::parse_str(&self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(records)
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

    async fn politician(&self, ctx: &Context<'_>) -> Result<Option<PoliticianResult>> {
        let politician_id = self.attributes["politicianId"].as_str();
        if let Some(politician_id) = politician_id {
            let politician_id = uuid::Uuid::parse_str(politician_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Politician::find_by_id(&db_pool, politician_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    async fn question(&self, ctx: &Context<'_>) -> Result<Option<QuestionResult>> {
        let question_id = self.attributes["questionId"].as_str();
        if let Some(question_id) = question_id {
            let question_id = uuid::Uuid::parse_str(question_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Question::find_by_id(&db_pool, question_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    async fn poll(&self, ctx: &Context<'_>) -> Result<Option<PollResult>> {
        let poll_id = self.attributes["pollId"].as_str();
        warn!("poll_id: {:?}", poll_id);
        if let Some(poll_id) = poll_id {
            let poll_id = uuid::Uuid::parse_str(poll_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Poll::find_by_id(&db_pool, poll_id).await?;
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
            embed_type: embed.embed_type,
            attributes: embed.attributes,
            created_at: embed.created_at,
            updated_at: embed.updated_at,
            created_by_id: embed.created_by.into(),
            updated_by_id: embed.updated_by.into(),
        }
    }
}
