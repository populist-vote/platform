use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{
        candidate_guide::CandidateGuide,
        enums::{EmbedType, RaceType, State, VoteType},
        race::Race,
    },
    Embed, Question,
};

use crate::context::ApiContext;

use super::{EmbedResult, OrganizationResult, QuestionResult, RaceResult};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CandidateGuideResult {
    id: ID,
    organization_id: ID,
    name: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[ComplexObject]
impl CandidateGuideResult {
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
                populist_url,
                embed_type AS "embed_type:EmbedType",
                attributes,
                created_at,
                created_by,
                updated_at,
                updated_by
                FROM embed
            WHERE embed_type = 'candidate_guide' 
            AND attributes->>'candidate_guide_id' = $1
        "#,
            self.id.as_str()
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(embeds.into_iter().map(EmbedResult::from).collect())
    }

    async fn races(&self, ctx: &Context<'_>) -> Result<Vec<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let races = sqlx::query_as!(
            Race,
            r#"
            SELECT 
            r.id,
            r.slug,
            r.title,
            r.office_id,
            r.race_type AS "race_type:RaceType", 
            r.vote_type AS "vote_type:VoteType", 
            r.party_id, 
            r.state AS "state:State",
            r.description,
            r.ballotpedia_link,
            r.early_voting_begins_date,
            r.winner_ids,
            r.total_votes,
            r.official_website,
            r.election_id,
            r.is_special_election,
            r.num_elect,
            r.created_at,
            r.updated_at 
            FROM candidate_guide_races 
            JOIN race r ON r.id = candidate_guide_races.race_id 
            WHERE candidate_guide_id = $1
        "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(races.into_iter().map(RaceResult::from).collect())
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
            name: c.name,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}
