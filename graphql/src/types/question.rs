use crate::context::ApiContext;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{DateTime, Question, QuestionSubmission, Respondent};

use super::SubmissionsOverTimeResult;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct QuestionResult {
    id: ID,
    prompt: String,
    response_char_limit: Option<i32>,
    response_placeholder_text: Option<String>,
    allow_anonymous_responses: bool,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct QuestionSubmissionResult {
    id: ID,
    respondent_id: Option<ID>,
    response: String,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ExternalUserResult {
    name: String,
    email: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct RespondentResult {
    name: String,
    email: String,
}

#[ComplexObject]
impl QuestionResult {
    async fn submissions(&self, ctx: &Context<'_>) -> Result<Vec<QuestionSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submissions = sqlx::query_as!(
            QuestionSubmission,
            r#"
                SELECT
                  id,
                  question_id,
                  respondent_id,
                  response,
                  created_at,
                  updated_at
                FROM question_submission
                WHERE question_id = $1
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
    ) -> Result<Vec<SubmissionsOverTimeResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submission_count_by_date = sqlx::query!(
            r#"
                SELECT
                  date_trunc('day', created_at) AS date,
                  COUNT(*) AS count
                FROM question_submission
                WHERE question_id = $1
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
            .map(|s| SubmissionsOverTimeResult {
                date: s.date.unwrap(),
                count: s.count.unwrap(),
            })
            .collect())
    }
}

#[ComplexObject]
impl QuestionSubmissionResult {
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
}

impl From<Question> for QuestionResult {
    fn from(q: Question) -> Self {
        Self {
            id: q.id.into(),
            prompt: q.prompt,
            response_char_limit: q.response_char_limit,
            response_placeholder_text: q.response_placeholder_text,
            allow_anonymous_responses: q.allow_anonymous_responses,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }
    }
}

impl From<QuestionSubmission> for QuestionSubmissionResult {
    fn from(q: QuestionSubmission) -> Self {
        Self {
            id: q.id.into(),
            respondent_id: q.respondent_id.map(|id| id.into()),
            response: q.response,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }
    }
}

impl From<Respondent> for RespondentResult {
    fn from(r: Respondent) -> Self {
        Self {
            name: r.name,
            email: r.email,
        }
    }
}
