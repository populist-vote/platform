use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::ElectionResult};
use async_graphql::*;
use db::{Election, UpsertElectionInput};

#[derive(Default)]
pub struct ElectionMutation;

#[derive(SimpleObject)]
struct DeleteElectionResult {
    id: String,
}

#[Object]
impl ElectionMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn upsert_election(
        &self,
        ctx: &Context<'_>,
        input: UpsertElectionInput,
    ) -> Result<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Election::upsert(&db_pool, &input).await?;
        Ok(ElectionResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_election(&self, ctx: &Context<'_>, id: String) -> Result<DeleteElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Election::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteElectionResult { id })
    }
}
