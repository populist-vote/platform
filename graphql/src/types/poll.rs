use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::poll::{Poll, PollOption},
    DateTime, PollSubmission, Respondent,
};

use crate::context::ApiContext;

use super::RespondentResult;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct PollResult {
    id: ID,
    name: Option<String>,
    prompt: String,
    embed_id: Option<ID>,
    allow_anonymous_responses: bool,
    allow_write_in_responses: bool,
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
    pub respondent_id: Option<ID>,
    pub poll_option_id: Option<ID>,
    pub write_in_response: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct SubmissionCountByDateResult {
    pub date: DateTime,
    pub count: i64,
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

    async fn submissions(&self, ctx: &Context<'_>) -> Result<Vec<PollSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submissions = sqlx::query_as!(
            PollSubmission,
            r#"
                SELECT
                  id,
                  poll_id,
                  poll_option_id,
                  write_in_response,
                  respondent_id,
                  created_at,
                  updated_at
                FROM poll_submission
                WHERE poll_id = $1
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(submissions.into_iter().map(|s| s.into()).collect())
    }

    async fn submission_count_by_date(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<SubmissionCountByDateResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submission_count_by_date = sqlx::query!(
            r#"
                SELECT
                  date_trunc('day', created_at) AS date,
                  COUNT(*) AS count
                FROM poll_submission
                WHERE poll_id = $1
                GROUP BY date
                ORDER BY date
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(submission_count_by_date
            .into_iter()
            .filter(|s| s.date.is_some())
            .map(|s| SubmissionCountByDateResult {
                date: s.date.unwrap(),
                count: s.count.unwrap(),
            })
            .collect())
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

    async fn respondent(&self, ctx: &Context<'_>) -> Result<Option<RespondentResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        if let Some(respondent_id) = self.respondent_id.clone() {
            let respondent = sqlx::query_as!(
                Respondent,
                r#"
                    SELECT
                      id,
                      name,
                      email,
                      created_at,
                      updated_at
                    FROM respondent
                    WHERE id = $1
                "#,
                uuid::Uuid::parse_str(respondent_id.as_str()).unwrap(),
            )
            .fetch_one(&db_pool)
            .await?;

            Ok(Some(respondent.into()))
        } else {
            Ok(None)
        }
    }

    async fn option(&self, ctx: &Context<'_>) -> Result<PollOptionResult> {
        if let Some(poll_option_id) = self.poll_option_id.clone() {
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let option = sqlx::query_as!(
                PollOption,
                r#"
                SELECT * FROM poll_option WHERE id = $1
                "#,
                uuid::Uuid::parse_str(&poll_option_id)?
            )
            .fetch_one(&db_pool)
            .await?;
            Ok(option.into())
        } else {
            Err("No poll option found".into())
        }
    }
}

impl From<Poll> for PollResult {
    fn from(p: Poll) -> Self {
        Self {
            id: p.id.into(),
            name: p.name,
            prompt: p.prompt,
            allow_anonymous_responses: p.allow_anonymous_responses,
            allow_write_in_responses: p.allow_write_in_responses,
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
            respondent_id: p.respondent_id.map(|id| id.into()),
            poll_option_id: p.poll_option_id.map(|id| id.into()),
            write_in_response: p.write_in_response,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
