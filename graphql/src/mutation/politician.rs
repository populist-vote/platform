use async_graphql::*;
use db::{CreatePoliticianInput, Politician, UpdatePoliticianInput};
use sqlx::{Pool, Postgres};

use crate::types::PoliticianResult;
#[derive(Default)]
pub struct PoliticianMutation;

#[derive(SimpleObject)]
struct DeletePoliticianResult {
    id: String,
}

#[Object]
impl PoliticianMutation {
    async fn create_politician(
        &self,
        ctx: &Context<'_>,
        input: CreatePoliticianInput,
    ) -> Result<PoliticianResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = Politician::create(db_pool, &input).await?;
        Ok(PoliticianResult::from(new_record))
    }

    async fn update_politician(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdatePoliticianInput,
    ) -> Result<PoliticianResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let updated_record =
            Politician::update(db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(PoliticianResult::from(updated_record))
    }

    async fn delete_politician(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeletePoliticianResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        Politician::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeletePoliticianResult { id })
    }
}
