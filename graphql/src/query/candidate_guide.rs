use async_graphql::{Context, Object, Result, ID};
use db::{models::candidate_guide::CandidateGuide, QuestionSubmission, Sentiment};

use crate::{
    context::ApiContext,
    types::{CandidateGuideResult, QuestionSubmissionResult},
};

#[derive(Default)]
pub struct CandidateGuideQuery;

#[Object]
impl CandidateGuideQuery {
    async fn candidate_guide_by_id(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<CandidateGuideResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record =
            CandidateGuide::find_by_id(&db_pool, uuid::Uuid::parse_str(id.as_str()).unwrap())
                .await?;
        Ok(record.into())
    }

    async fn candidate_guides_by_organization(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
    ) -> Result<Vec<CandidateGuideResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = CandidateGuide::find_by_organization(
            &db_pool,
            uuid::Uuid::parse_str(organization_id.as_str()).unwrap(),
        )
        .await?;
        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    async fn recent_candidate_guide_question_submissions(
        &self,
        ctx: &Context<'_>,
        candidate_guide_id: ID,
        limit: Option<i64>,
    ) -> Result<Vec<QuestionSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            QuestionSubmission,
            r#"
            SELECT
                qs.id,
                qs.candidate_id,
                qs.question_id,
                qs.response,
                qs.sentiment AS "sentiment:Sentiment",
                qs.created_at,
                qs.updated_at,
                qs.respondent_id,
                qs.is_locked
            FROM
                question_submission qs
            JOIN
                candidate_guide_questions cgq ON qs.question_id = cgq.question_id
            WHERE
                cgq.candidate_guide_id = $1
            ORDER BY
                qs.created_at DESC
            LIMIT $2;
            "#,
            uuid::Uuid::parse_str(candidate_guide_id.as_str())?,
            limit.unwrap_or(10),
        )
        .fetch_all(&db_pool)
        .await?;
        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    /// Returns number of submissions per set of questions in a candidate guide
    async fn candidate_guide_submission_count(
        &self,
        ctx: &Context<'_>,
        candidate_guide_id: ID,
    ) -> Result<i64> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT qs.question_id) AS total_submissions, COUNT (DISTINCT cgq.question_id) AS total_questions
            FROM candidate_guide cg
            JOIN candidate_guide_questions cgq ON cg.id = cgq.candidate_guide_id
            JOIN question_submission qs ON cgq.question_id = qs.question_id
            WHERE cg.id = $1;
            "#,
            uuid::Uuid::parse_str(candidate_guide_id.as_str())?,
        )
        .fetch_one(&db_pool)
        .await?;
        let total_submissions = result.total_submissions.unwrap_or(0);
        let total_questions = result.total_questions.unwrap_or(0);
        let count = if total_questions > 0 {
            total_submissions as i64 / total_questions as i64
        } else {
            0
        };
        Ok(count)
    }
}
