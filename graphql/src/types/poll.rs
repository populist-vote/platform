use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::poll::{Poll, PollOption},
    DateTime,
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
