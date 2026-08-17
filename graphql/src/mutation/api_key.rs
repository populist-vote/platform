use crate::{
    api_key::interactive_user_id,
    context::ApiContext,
    types::{CreatedApiKeyResult, Error},
};
use async_graphql::{Context, Object, Result, ID};
use auth::generate_api_key;
use db::ApiKey;

#[derive(Default)]
pub struct ApiKeyMutation;

#[Object]
impl ApiKeyMutation {
    /// Creates an API key for the current registered user.
    /// The secret is returned once and only its hash is stored.
    async fn create_api_key(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<CreatedApiKeyResult, Error> {
        let user_id = interactive_user_id(ctx)?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 100 {
            return Err(Error::BadInput {
                field: "name".to_string(),
                message: "API key names must contain between 1 and 100 characters".to_string(),
            });
        }

        let generated = generate_api_key();
        let pool = &ctx.data::<ApiContext>().unwrap().pool;
        let api_key = ApiKey::create(
            pool,
            user_id,
            name,
            &generated.display_prefix,
            &generated.hash,
        )
        .await?;

        Ok(CreatedApiKeyResult {
            api_key: api_key.into(),
            key: generated.plaintext,
        })
    }

    /// Immediately revokes an API key owned by the current registered user.
    async fn revoke_api_key(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        let user_id = interactive_user_id(ctx)?;
        let id = uuid::Uuid::parse_str(id.as_str())?;
        let pool = &ctx.data::<ApiContext>().unwrap().pool;
        Ok(ApiKey::revoke(pool, user_id, id).await?.is_some())
    }
}
