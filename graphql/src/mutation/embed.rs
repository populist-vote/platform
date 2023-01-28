use async_graphql::{Context, Object, Result};
use db::{Embed, UpsertEmbedInput};

use crate::{context::ApiContext, types::EmbedResult};

#[derive(Default)]
pub struct EmbedMutation;

#[Object]
impl EmbedMutation {
    // TODO: New guard to check organization_id (and role ultimately)
    async fn upsert_embed(
        &self,
        ctx: &Context<'_>,
        input: UpsertEmbedInput,
    ) -> Result<EmbedResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Embed::upsert(&db_pool, &input).await?;
        Ok(EmbedResult::from(new_record))
    }
}
