use crate::is_admin;
use async_graphql::{Context, Object, Result, SimpleObject, ID};
use db::UpsertPollInput;

use crate::{context::ApiContext, types::PollResult};

#[derive(Default)]
pub struct PollMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeletePollResult {
    id: ID,
}

#[Object]
impl PollMutation {
    async fn upsert_poll(&self, ctx: &Context<'_>, input: UpsertPollInput) -> Result<PollResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_poll = db::Poll::upsert(&db_pool, &input).await?;
        Ok(new_poll.into())
    }

    async fn delete_poll(&self, ctx: &Context<'_>, id: ID) -> Result<DeletePollResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        sqlx::query!(
            r#"
                DELETE FROM poll_option WHERE poll_id = $1
            "#,
            uuid::Uuid::parse_str(&id)?
        )
        .execute(&db_pool)
        .await?;
        Ok(DeletePollResult { id })
    }
}
