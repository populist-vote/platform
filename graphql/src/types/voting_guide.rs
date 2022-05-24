use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use db::{models::voting_guide::VotingGuide, Politician, Election};

use crate::context::ApiContext;

use super::{PoliticianResult, ElectionResult};

#[derive(InputObject)]
pub struct UpsertVotingGuideInput {
    pub id: Option<ID>,
    pub election_id: ID,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpsertVotingGuideCandidateInput {
    pub voting_guide_id: ID,
    pub candidate_id: ID,
    pub is_endorsement: Option<bool>,
    pub note: Option<String>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct VotingGuideResult {
    id: ID,
    user_id: ID,
    election_id: ID,
    title: Option<String>,
    description: Option<String>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct VotingGuideCandidateResult {
    pub candidate_id: ID,
    pub is_endorsement: bool,
    pub note: Option<String>,
}

#[ComplexObject]
impl VotingGuideResult {
    async fn election(&self, ctx: &Context<'_>) -> Result<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let election = Election::find_by_id(&db_pool, uuid::Uuid::parse_str(self.election_id.as_str()).unwrap()).await?;
        Ok(election.into())
    }
    
    async fn candidates(&self, ctx: &Context<'_>) -> Result<Vec<VotingGuideCandidateResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let records = sqlx::query!(
            r#"
                SELECT
                    candidate_id,
                    is_endorsement,
                    note
                FROM
                    voting_guide_candidates
                WHERE
                    voting_guide_id = $1
            "#,
            uuid::Uuid::parse_str(self.id.clone().as_str()).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        let results = records
            .into_iter()
            .map(|record| VotingGuideCandidateResult {
                candidate_id: record.candidate_id.into(),
                is_endorsement: record.is_endorsement,
                note: record.note,
            })
            .collect();
        Ok(results)
    }
}

#[ComplexObject]
impl VotingGuideCandidateResult {
    async fn politician(&self, ctx: &Context<'_>) -> Result<PoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Politician::find_by_id(
            &db_pool,
            uuid::Uuid::parse_str(self.candidate_id.clone().as_str()).unwrap(),
        )
        .await?;

        Ok(record.into())
    }
}

impl From<VotingGuide> for VotingGuideResult {
    fn from(record: VotingGuide) -> Self {
        VotingGuideResult {
            id: record.id.into(),
            user_id: record.user_id.into(),
            election_id: record.election_id.into(),
            title: record.title,
            description: record.description,
        }
    }
}
