use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{
        candidate_guide::CandidateGuide,
        enums::{RaceType, State, VoteType},
    },
    Embed, EmbedType, Question,
};

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
    race: RaceResult,
    were_candidates_emailed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
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

    async fn races(&self, ctx: &Context<'_>) -> Result<Vec<CandidateGuideRaceResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let races = sqlx::query!(
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
            r.num_precincts_reporting,
            r.total_precincts,
            r.official_website,
            r.election_id,
            r.is_special_election,
            r.num_elect,
            r.created_at,
            r.updated_at,
            cgr.were_candidates_emailed,
            cgr.created_at AS cgr_created_at,
            cgr.updated_at AS cgr_updated_at
            FROM candidate_guide_races cgr
            JOIN race r ON r.id = cgr.race_id 
            WHERE candidate_guide_id = $1
        "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        let results = races
            .iter()
            .map(|r| CandidateGuideRaceResult {
                race: RaceResult {
                    id: r.id.into(),
                    slug: r.slug.clone(),
                    title: r.title.clone(),
                    office_id: r.office_id.into(),
                    race_type: r.race_type.clone(),
                    vote_type: r.vote_type.clone(),
                    party_id: r.party_id.map(|p| p.into()),
                    state: r.state.clone(),
                    description: r.description.clone(),
                    ballotpedia_link: r.ballotpedia_link.clone(),
                    early_voting_begins_date: r.early_voting_begins_date,
                    official_website: r.official_website.clone(),
                    election_id: r.election_id.map(|e| e.into()),
                    is_special_election: r.is_special_election,
                    num_elect: r.num_elect,
                },
                were_candidates_emailed: r.were_candidates_emailed.unwrap_or(false),
                created_at: r.cgr_created_at,
                updated_at: r.cgr_updated_at,
            })
            .collect::<Vec<CandidateGuideRaceResult>>();

        Ok(results)
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
