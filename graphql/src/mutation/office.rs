use async_graphql::{Context, Object, Result, SimpleObject};
use db::{CreateOfficeInput, Office, UpdateOfficeInput};

use crate::{context::ApiContext, mutation::StaffOnly, types::OfficeResult};

#[derive(Default)]
pub struct OfficeMutation;

#[derive(SimpleObject)]
struct DeleteOfficeResult {
    id: String,
}

#[Object]
impl OfficeMutation {
    #[graphql(guard = "StaffOnly")]
    async fn create_office(
        &self,
        ctx: &Context<'_>,
        input: CreateOfficeInput,
    ) -> Result<OfficeResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_office = Office::create(&db_pool, &input).await?;
        Ok(new_office.into())
    }

    #[graphql(guard = "StaffOnly")]
    async fn update_office(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateOfficeInput,
    ) -> Result<OfficeResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_office = Office::update(&db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(updated_office.into())
    }

    #[graphql(guard = "StaffOnly")]
    async fn delete_office(&self, ctx: &Context<'_>, id: String) -> Result<DeleteOfficeResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Office::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteOfficeResult { id })
    }
}
