use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::BallotMeasureResult};
use async_graphql::*;
use db::{BallotMeasure, UpsertBallotMeasureInput};
#[derive(Default)]
pub struct BallotMeasureMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteBallotMeasureResult {
    id: String,
}

#[Object]
impl BallotMeasureMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn upsert_ballot_measure(
        &self,
        ctx: &Context<'_>,
        input: UpsertBallotMeasureInput,
    ) -> Result<BallotMeasureResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = BallotMeasure::upsert(&db_pool, &input).await?;
        Ok(BallotMeasureResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_ballot_measure(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeleteBallotMeasureResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        BallotMeasure::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteBallotMeasureResult { id })
    }
}
