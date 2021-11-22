use async_graphql::*;
use db::{Argument, UpdateArgumentInput};
use sqlx::{Pool, Postgres};

use crate::types::ArgumentResult;
#[derive(Default)]
pub struct ArgumentMutation;

#[derive(SimpleObject)]
struct DeleteArgumentResult {
    id: String,
}

#[Object]
impl ArgumentMutation {
    async fn update_argument(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateArgumentInput,
    ) -> Result<ArgumentResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let updated_record = Argument::update(db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(ArgumentResult::from(updated_record))
    }

    async fn delete_argument(&self, ctx: &Context<'_>, id: String) -> Result<DeleteArgumentResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        Argument::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteArgumentResult { id })
    }
}
