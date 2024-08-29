use crate::types::QuestionResult;
use crate::{context::ApiContext, types::QuestionSubmissionResult};
use async_graphql::{Context, Object, Result, ID};
use db::{Question, QuestionSubmission, Sentiment};

#[derive(Default)]
pub struct QuestionQuery;

#[derive(Default)]
pub struct QuestionSubmissionQuery;

#[Object]
impl QuestionQuery {
    async fn question_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<QuestionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Question::find_by_id(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(record.into())
    }
}

#[Object]
impl QuestionSubmissionQuery {
    async fn related_question_submission_by_candidate_and_question(
        &self,
        ctx: &Context<'_>,
        candidate_id: ID,
        question_id: ID,
        organization_id: ID,
    ) -> Result<Option<QuestionSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            QuestionSubmission,
            r#"
                SELECT
                  qs.id,
                  qs.question_id,
                  qs.respondent_id,
                  qs.candidate_id,
                  qs.response,
                  qs.editorial,
                  qs.translations,
                  qs.sentiment AS "sentiment: Sentiment",
                  qs.copied_from_id,
                  qs.created_at,
                  qs.updated_at
                FROM
                    question_submission qs
                    JOIN question q ON q.id = qs.question_id
                WHERE
                    qs.candidate_id = $1
                    AND(q.prompt = (SELECT prompt FROM question WHERE id = $2)
                        OR SIMILARITY (q.prompt,  (SELECT prompt FROM question WHERE id = $2)) > 0.95)
                    AND q.organization_id = $3
                    AND q.id != $2
                LIMIT 1;
            "#,
            uuid::Uuid::parse_str(candidate_id.as_str())?,
            uuid::Uuid::parse_str(question_id.as_str())?,
            uuid::Uuid::parse_str(organization_id.as_str())?,
        )
        .fetch_optional(&db_pool)
        .await?;

        Ok(record.map(|r| r.into()))
    }
}
