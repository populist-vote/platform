use async_graphql::{Context, Object, Result};
use db::models::candidate_guide::{CandidateGuide, UpsertCandidateGuideInput};

use crate::{context::ApiContext, types::CandidateGuideResult};

#[derive(Default)]
pub struct CandidateGuideMutation;

#[Object]
impl CandidateGuideMutation {
    async fn upsert_candidate_guide(
        &self,
        ctx: &Context<'_>,
        input: UpsertCandidateGuideInput,
    ) -> Result<CandidateGuideResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let upsert = CandidateGuide::upsert(&db_pool, &input).await?;
        Ok(upsert.into())
    }

    async fn delete_candidate_guide(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        CandidateGuide::delete(&db_pool, uuid::Uuid::parse_str(id.as_str()).unwrap()).await?;
        Ok(true)
    }
}
