use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::BillResult};
use async_graphql::*;
use db::{Bill, CreateArgumentInput, CreateBillInput, UpdateBillInput};
use sqlx::{Pool, Postgres};
#[derive(Default)]
pub struct BillMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
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
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn create_bill(&self, ctx: &Context<'_>, input: CreateBillInput) -> Result<BillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Bill::create(&db_pool, &input).await?;
        if input.arguments.is_some() {
            handle_nested_arguments(&db_pool, new_record.id, input.arguments.unwrap()).await?;
        }
        Ok(BillResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn update_bill(
        &self,
        ctx: &Context<'_>,
        id: Option<String>,
        legiscan_bill_id: Option<i32>,
        input: UpdateBillInput,
    ) -> Result<BillResult> {
        if id.is_none() && legiscan_bill_id.is_none() {
            panic!("Please provide a populist bill ID or legiscan bill id")
        }
        let id = id.map(|id| uuid::Uuid::parse_str(&id).unwrap());
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_record = Bill::update(&db_pool, id, legiscan_bill_id, &input).await?;
        if input.arguments.is_some() {
            handle_nested_arguments(&db_pool, updated_record.id, input.arguments.unwrap()).await?;
        }
        Ok(BillResult::from(updated_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_bill(&self, ctx: &Context<'_>, id: String) -> Result<DeleteBillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Bill::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteBillResult { id })
    }
}
