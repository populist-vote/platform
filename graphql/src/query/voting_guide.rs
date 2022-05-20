use async_graphql::{Context, Object, Result};
use db::models::voting_guide::VotingGuide;

use crate::{context::ApiContext, types::VotingGuideResult};

#[derive(Default)]
pub struct VotingGuideQuery;

#[Object]
impl VotingGuideQuery {
    async fn voting_guide_by_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Voting guide id")] id: String,
    ) -> Result<VotingGuideResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = VotingGuide::find_by_id(&db_pool, uuid::Uuid::parse_str(&id).unwrap()).await?;

        Ok(record.into())
    }
}
