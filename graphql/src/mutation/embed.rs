use async_graphql::{Context, Object, Result, SimpleObject};
use auth::Claims;
use db::{Embed, UpsertEmbedInput};
use jsonwebtoken::TokenData;

use crate::{context::ApiContext, is_admin, types::EmbedResult};

#[derive(Default)]
pub struct EmbedMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteEmbedResult {
    id: String,
}

#[Object]
impl EmbedMutation {
    #[graphql(visible = "is_admin")]
    async fn upsert_embed(
        &self,
        ctx: &Context<'_>,
        input: UpsertEmbedInput,
    ) -> Result<EmbedResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_org_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .organization_id;

        if let Some(embed_id) = input.id {
            let existing_embed_org_id =
                Embed::find_by_id(&db_pool, embed_id).await?.organization_id;
            if existing_embed_org_id != input.organization_id.unwrap() {
                return Err("Unauthorized".into());
            }
            if existing_embed_org_id != user_org_id.unwrap() {
                return Err("Unauthorized".into());
            }
        }
        let updated_by = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;

        let upserted_record = Embed::upsert(&db_pool, &input, &updated_by).await?;
        Ok(EmbedResult::from(upserted_record))
    }

    #[graphql(visible = "is_admin")]
    async fn delete_embed(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<DeleteEmbedResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_org_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .organization_id;

        let existing_embed_org_id = Embed::find_by_id(&db_pool, id).await?.organization_id;
        if existing_embed_org_id != user_org_id.unwrap() {
            return Err("Unauthorized".into());
        }

        Embed::delete(&db_pool, id).await?;
        Ok(DeleteEmbedResult { id: id.to_string() })
    }
}
