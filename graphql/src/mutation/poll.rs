use crate::{is_admin, types::PollSubmissionResult};
use async_graphql::{Context, Object, Result, SimpleObject, ID};
use db::{Respondent, UpsertPollInput, UpsertPollSubmissionInput, UpsertRespondentInput};

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
        let upserted_poll = db::Poll::upsert(&db_pool, &input).await?;
        Ok(upserted_poll.into())
    }

    async fn upsert_poll_submission(
        &self,
        ctx: &Context<'_>,
        respondent_input: Option<UpsertRespondentInput>,
        poll_submission_input: UpsertPollSubmissionInput,
    ) -> Result<PollSubmissionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let respondent = match respondent_input {
            Some(respondent_input) => {
                Some(db::Respondent::upsert(&db_pool, &respondent_input).await?)
            }
            None => None,
        };
        let poll_submission_input = UpsertPollSubmissionInput {
            respondent_id: respondent.map(|r| r.id),
            ..poll_submission_input
        };
        let poll = db::PollSubmission::upsert(&db_pool, &poll_submission_input).await?;
        Ok(poll.into())
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
