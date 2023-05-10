use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::poll::{Poll, PollOption},
    DateTime, PollSubmission,
};

use crate::context::ApiContext;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct PollResult {
    id: ID,
    name: Option<String>,
    prompt: String,
    embed_id: Option<ID>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct PollOptionResult {
    id: ID,
    poll_id: ID,
    option_text: String,
    is_write_in: bool,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct PollSubmissionResult {
    pub id: ID,
    pub poll_id: ID,
    pub respondent_id: ID,
    pub poll_option_id: ID,
    pub write_in_response: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[ComplexObject]
impl PollResult {
    async fn options(&self, ctx: &Context<'_>) -> Result<Vec<PollOptionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let options = sqlx::query_as!(
            PollOption,
            r#"
                SELECT * FROM poll_option WHERE poll_id = $1
            "#,
            uuid::Uuid::parse_str(&self.id)?
        )
        .fetch_all(&db_pool)
        .await?
        .into_iter()
        .map(|o| o.into())
        .collect();
        Ok(options)
    }
}

#[ComplexObject]
impl PollSubmissionResult {
    async fn poll(&self, ctx: &Context<'_>) -> Result<PollResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let poll = sqlx::query_as!(
            Poll,
            r#"
                SELECT * FROM poll WHERE id = $1
            "#,
            uuid::Uuid::parse_str(&self.poll_id)?
        )
        .fetch_one(&db_pool)
        .await?;
        Ok(poll.into())
    }
}

impl From<Poll> for PollResult {
    fn from(p: Poll) -> Self {
        Self {
            id: p.id.into(),
            name: p.name,
            prompt: p.prompt,
            embed_id: p.embed_id.map(|id| id.into()),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

impl From<PollOption> for PollOptionResult {
    fn from(p: PollOption) -> Self {
        Self {
            id: p.id.into(),
            poll_id: p.poll_id.into(),
            option_text: p.option_text,
            is_write_in: p.is_write_in,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

impl From<PollSubmission> for PollSubmissionResult {
    fn from(p: PollSubmission) -> Self {
        Self {
            id: p.id.into(),
            poll_id: p.poll_id.into(),
            respondent_id: p.respondent_id.into(),
            poll_option_id: p.poll_option_id.into(),
            write_in_response: p.write_in_response,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
