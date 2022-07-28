use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::ElectionResult};
use async_graphql::*;
use db::{CreateElectionInput, Election, UpdateElectionInput};

#[derive(Default)]
pub struct ElectionMutation;

#[derive(SimpleObject)]
struct DeleteElectionResult {
    id: String,
}

#[Object]
impl ElectionMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn create_election(
        &self,
        ctx: &Context<'_>,
        input: CreateElectionInput,
    ) -> Result<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Election::create(&db_pool, &input).await?;
        Ok(ElectionResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn update_election(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateElectionInput,
    ) -> Result<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_record =
            Election::update(&db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(ElectionResult::from(updated_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_election(&self, ctx: &Context<'_>, id: String) -> Result<DeleteElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Election::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteElectionResult { id })
    }
}
