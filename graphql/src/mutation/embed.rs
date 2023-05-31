use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use auth::Claims;
use db::{DateTime, Embed, UpsertEmbedInput};
use jsonwebtoken::TokenData;

use crate::{
    context::ApiContext,
    is_admin,
    types::{EmbedOriginResult, EmbedResult},
};

#[derive(Default)]
pub struct EmbedMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteEmbedResult {
    id: String,
}

#[derive(InputObject)]
#[graphql(visible = "is_admin")]
struct PingEmbedOriginInput {
    embed_id: uuid::Uuid,
    url: String,
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

    async fn ping_embed_origin(
        &self,
        ctx: &Context<'_>,
        input: PingEmbedOriginInput,
    ) -> Result<EmbedOriginResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            EmbedOriginResult,
            r#"
            INSERT INTO embed_origin (embed_id, url)
            VALUES ($1, $2)
            ON CONFLICT (embed_id, url)
            DO UPDATE SET last_ping_at = CURRENT_TIMESTAMP
            RETURNING url, last_ping_at as "last_ping_at: DateTime"
        "#,
            input.embed_id,
            input.url
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(record)
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
