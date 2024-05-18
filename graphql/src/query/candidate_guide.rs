use async_graphql::{Context, Object, Result, ID};
use db::models::candidate_guide::CandidateGuide;

use crate::{context::ApiContext, types::CandidateGuideResult};

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
            CandidateGuide::find_by_id(&db_pool, uuid::Uuid::parse_str(&id.as_str()).unwrap())
                .await?;
        Ok(record.into())
    }
}
