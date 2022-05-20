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

    async fn voting_guides_by_user_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "User id")] user_id: String,
    ) -> Result<Vec<VotingGuideResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records =
            VotingGuide::find_by_user_id(&db_pool, uuid::Uuid::parse_str(&user_id).unwrap())
                .await?;

        Ok(records.into_iter().map(|record| record.into()).collect())
    }
}
