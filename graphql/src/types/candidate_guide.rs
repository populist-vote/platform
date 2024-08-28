use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{models::candidate_guide::CandidateGuide, Embed, EmbedType, Question};

use crate::context::ApiContext;

use super::{EmbedResult, OrganizationResult, QuestionResult, RaceResult};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CandidateGuideResult {
    id: ID,
    organization_id: ID,
    name: Option<String>,
    submissions_open_at: Option<chrono::DateTime<chrono::Utc>>,
    submissions_close_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(SimpleObject)]
pub struct CandidateGuideRaceResult {
    pub race: RaceResult,
    pub were_candidates_emailed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[ComplexObject]
impl CandidateGuideResult {
    async fn embed_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) AS count
            FROM embed
            WHERE embed_type = 'candidate_guide' 
            AND attributes->>'candidateGuideId' = $1
        "#,
            self.id.as_str()
        )
        .fetch_one(&db_pool)
        .await?;
        Ok(result.count.unwrap_or(0))
    }

    async fn embeds(&self, ctx: &Context<'_>) -> Result<Vec<EmbedResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let embeds = sqlx::query_as!(
            Embed,
            r#"
            SELECT 
                id,
                organization_id,
                name,
                description,
                embed_type AS "embed_type:EmbedType",
                attributes,
                created_at,
                created_by,
                updated_at,
                updated_by
                FROM embed
            WHERE embed_type = 'candidate_guide' 
            AND attributes->>'candidateGuideId' = $1
        "#,
            self.id.as_str()
        )
        .fetch_all(&db_pool)
        .await?;

        tracing::warn!("embeds: {:?}", embeds);

        Ok(embeds.into_iter().map(EmbedResult::from).collect())
    }

    async fn organization(&self, ctx: &Context<'_>) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let organization = db::Organization::find_by_id(
            &db_pool,
            uuid::Uuid::parse_str(self.organization_id.as_str()).unwrap(),
        )
        .await?;
        Ok(organization.into())
    }

    async fn questions(&self, ctx: &Context<'_>) -> Result<Vec<QuestionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let questions = sqlx::query_as!(
            Question,
            r#"
                SELECT
                  id,
                  prompt,
                  translations,
                  response_char_limit,
                  response_placeholder_text,
                  allow_anonymous_responses,
                  embed_id,
                  organization_id,
                  created_at,
                  updated_at
                FROM question
                JOIN candidate_guide_questions ON question.id = candidate_guide_questions.question_id
                WHERE candidate_guide_id = $1
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;
        Ok(questions.into_iter().map(QuestionResult::from).collect())
    }

    /// Returns the total number of question submissions in the candidate guide divided by the number of questions
    /// in the candidate guide to get the number of intake submissions per candidate guide.
    async fn submission_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT qs.id) AS total_submissions, COUNT (DISTINCT cgq.question_id) AS total_questions
            FROM candidate_guide cg
            JOIN candidate_guide_questions cgq ON cg.id = cgq.candidate_guide_id
            JOIN question_submission qs ON cgq.question_id = qs.question_id
            WHERE 
                cg.id = $1
                AND qs.response IS NOT NULL
                AND qs.response != ''
            "#,
            uuid::Uuid::parse_str(&self.id.as_str())?,
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

impl From<CandidateGuide> for CandidateGuideResult {
    fn from(c: CandidateGuide) -> Self {
        Self {
            id: ID::from(c.id),
            organization_id: ID::from(c.organization_id),
            name: c.name,
            submissions_open_at: c.submissions_open_at,
            submissions_close_at: c.submissions_close_at,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}
