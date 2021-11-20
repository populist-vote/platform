use async_graphql::*;
use db::{CreateElectionInput, Election, UpdateElectionInput};
use sqlx::{Pool, Postgres};

use crate::types::ElectionResult;
#[derive(Default)]
pub struct ElectionMutation;

#[derive(SimpleObject)]
struct DeleteElectionResult {
    id: String,
}

#[Object]
impl ElectionMutation {
    async fn create_election(
        &self,
        ctx: &Context<'_>,
        input: CreateElectionInput,
    ) -> Result<ElectionResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = Election::create(db_pool, &input).await?;
        Ok(ElectionResult::from(new_record))
    }

    async fn update_election(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateElectionInput,
    ) -> Result<ElectionResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let updated_record = Election::update(db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(ElectionResult::from(updated_record))
    }

    async fn delete_election(&self, ctx: &Context<'_>, id: String) -> Result<DeleteElectionResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        Election::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteElectionResult { id })
    }
}
