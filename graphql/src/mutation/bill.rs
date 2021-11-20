use async_graphql::*;
use db::{Bill, CreateArgumentInput, CreateBillInput, UpdateBillInput};
use sqlx::{Pool, Postgres};

use crate::types::BillResult;
#[derive(Default)]
pub struct BillMutation;

#[derive(SimpleObject)]
struct DeleteBillResult {
    id: String,
}

async fn handle_nested_arguments(
    db_pool: &Pool<Postgres>,
    bill_id: uuid::Uuid,
    arguments_input: Vec<CreateArgumentInput>,
) -> Result<(), Error> {
    if !arguments_input.is_empty() {
        for input in arguments_input {
            Bill::create_bill_argument(
                db_pool,
                bill_id,
                uuid::Uuid::parse_str(&input.author_id)?,
                &input,
            )
            .await?;
        }
    }
    Ok(())
}

#[Object]
impl BillMutation {
    async fn create_bill(&self, ctx: &Context<'_>, input: CreateBillInput) -> Result<BillResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = Bill::create(db_pool, &input).await?;
        if input.arguments.is_some() {
            handle_nested_arguments(db_pool, new_record.id, input.arguments.unwrap()).await?;
        }
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
        if input.arguments.is_some() {
            handle_nested_arguments(db_pool, updated_record.id, input.arguments.unwrap()).await?;
        }
        Ok(BillResult::from(updated_record))
    }

    async fn delete_bill(&self, ctx: &Context<'_>, id: String) -> Result<DeleteBillResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        Bill::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteBillResult { id })
    }
}
