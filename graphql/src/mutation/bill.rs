use async_graphql::*;
use db::{Bill, CreateBillInput, UpdateBillInput};
use sqlx::{Pool, Postgres};

use crate::types::BillResult;
#[derive(Default)]
pub struct BillMutation;

#[derive(SimpleObject)]
struct DeleteBillResult {
    id: String,
}

#[Object]
impl BillMutation {
    async fn create_bill(&self, ctx: &Context<'_>, input: CreateBillInput) -> Result<BillResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = Bill::create(db_pool, &input).await?;
        Ok(BillResult::from(new_record))
    }

    async fn update_bill(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateBillInput,
    ) -> Result<BillResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let updated_record = Bill::update(db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(BillResult::from(updated_record))
    }

    async fn delete_bill(&self, ctx: &Context<'_>, id: String) -> Result<DeleteBillResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        Bill::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteBillResult { id })
    }
}
