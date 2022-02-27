use async_graphql::*;
use db::{BallotMeasure, CreateBallotMeasureInput, UpdateBallotMeasureInput};

use crate::{context::ApiContext, types::BallotMeasureResult};
#[derive(Default)]
pub struct BallotMeasureMutation;

#[derive(SimpleObject)]
struct DeleteBallotMeasureResult {
    id: String,
}

#[Object]
impl BallotMeasureMutation {
    async fn create_ballot_measure(
        &self,
        ctx: &Context<'_>,
        election_id: uuid::Uuid,
        input: CreateBallotMeasureInput,
    ) -> Result<BallotMeasureResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = BallotMeasure::create(&db_pool, election_id, &input).await?;
        Ok(BallotMeasureResult::from(new_record))
    }

    async fn update_ballot_measure(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateBallotMeasureInput,
    ) -> Result<BallotMeasureResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_record =
            BallotMeasure::update(&db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(BallotMeasureResult::from(updated_record))
    }

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
