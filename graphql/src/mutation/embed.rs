use std::collections::HashMap;

use async_graphql::{Context, Error, InputObject, Object, Result, SimpleObject};
use auth::AccessTokenClaims;
use config::Config;
use db::{DateTime, Embed, UpsertEmbedInput};
use jsonwebtoken::TokenData;
use url::{Position, Url};

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
        let user_org_id = input.organization_id;

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

        let user = ctx.data::<Option<TokenData<AccessTokenClaims>>>()?;
        let user_id = match user {
            Some(u) => Some(u.claims.sub),
            None => return Err("Unauthorized".into()),
        };

        let upserted_record = Embed::upsert(&db_pool, &input, &user_id.unwrap()).await?;
        Ok(EmbedResult::from(upserted_record))
    }

    #[graphql(visible = "is_admin")]
    async fn ping_embed_origin(
        &self,
        ctx: &Context<'_>,
        input: PingEmbedOriginInput,
    ) -> Result<EmbedOriginResult> {
        let cleaned = parse_url_and_retain_token_param(&input.url).ok_or("Invalid URL")?;
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        match Config::is_allowed_origin(&cleaned) {
            true => {
                let record = sqlx::query_as!(
                    EmbedOriginResult,
                    r#"
                    INSERT INTO embed_origin (embed_id, url)
                    VALUES ($1, $2)
                    ON CONFLICT (embed_id, url)
                    DO UPDATE SET last_ping_at = CURRENT_TIMESTAMP
                    RETURNING url, last_ping_at as "last_ping_at: DateTime", page_title
                "#,
                    input.embed_id,
                    cleaned
                )
                .fetch_one(&db_pool)
                .await?;

                Ok(record)
            }
            _ => Err("URL is not an allowed Populist origin".into()),
        }
    }

    // Needs an org guard
    #[graphql(visible = "is_admin")]
    async fn delete_embed(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<DeleteEmbedResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        Embed::delete(&db_pool, id).await?;
        Ok(DeleteEmbedResult { id: id.to_string() })
    }
}

fn parse_url_and_retain_token_param(input_url: &str) -> Option<String> {
    // Parse the URL
    let parsed = match Url::parse(input_url) {
        Ok(u) => u,
        Err(_) => return None,
    };

    // Retrieve query parameters and convert them to a HashMap
    let query_params: HashMap<_, _> = parsed.query_pairs().into_owned().collect();

    if let Some(token) = query_params.get("token") {
        // Create a new URL with only the "token" parameter
        let new_url =
            Url::parse_with_params(&parsed[..Position::AfterPath], vec![("token", token)]).ok()?;
        Some(new_url.to_string())
    } else {
        Some(parsed[..Position::AfterPath].to_string())
    }
}

#[test]
fn parse_url_and_retain_token_param_test() {
    let url = "https://www.youtube.com/watch?v=12345&token=abcde&extra=123";
    let new_url = parse_url_and_retain_token_param(url);
    assert_eq!(
        new_url,
        Some("https://www.youtube.com/watch?token=abcde".to_string())
    );

    let url = "https://www.youtube.com/watch";
    let new_url = parse_url_and_retain_token_param(url);
    assert_eq!(new_url, Some("https://www.youtube.com/watch".to_string()));
}
