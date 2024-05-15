use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{models::candidate_guide::CandidateGuide, Question};

use crate::context::ApiContext;

use super::{OrganizationResult, QuestionResult, RaceResult};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CandidateGuideResult {
    id: ID,
    race_id: ID,
    organization_id: ID,
    name: Option<String>,
}

#[ComplexObject]
impl CandidateGuideResult {
    async fn race(&self, ctx: &Context<'_>) -> Result<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let race = db::Race::find_by_id(
            &db_pool,
            uuid::Uuid::parse_str(&self.race_id.as_str()).unwrap(),
        )
        .await?;
        Ok(race.into())
    }

    async fn organization(&self, ctx: &Context<'_>) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let organization = db::Organization::find_by_id(
            &db_pool,
            uuid::Uuid::parse_str(&self.organization_id.as_str()).unwrap(),
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
                  response_char_limit,
                  response_placeholder_text,
                  allow_anonymous_responses,
                  embed_id,
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
}

impl From<CandidateGuide> for CandidateGuideResult {
    fn from(c: CandidateGuide) -> Self {
        Self {
            id: ID::from(c.id),
            organization_id: ID::from(c.organization_id),
            race_id: c.race_id.map(ID::from).unwrap_or_default(),
            name: c.name,
        }
    }
}
