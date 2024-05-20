use async_graphql::{Context, Object, Result};
use auth::AccessTokenClaims;
use db::models::candidate_guide::{CandidateGuide, UpsertCandidateGuideInput};
use jsonwebtoken::TokenData;

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
        let user = ctx.data::<Option<TokenData<AccessTokenClaims>>>().unwrap();
        let organization_id = user.as_ref().unwrap().claims.organization_id.unwrap();
        let input = UpsertCandidateGuideInput {
            user_id: Some(user.as_ref().unwrap().claims.sub),
            organization_id: Some(organization_id),
            ..input
        };
        let upsert = CandidateGuide::upsert(&db_pool, &input).await?;
        Ok(upsert.into())
    }

    async fn delete_candidate_guide(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        CandidateGuide::delete(&db_pool, uuid::Uuid::parse_str(id.as_str()).unwrap()).await?;
        Ok(true)
    }
}
