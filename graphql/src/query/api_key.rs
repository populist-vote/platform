use crate::{
    api_key::interactive_user_id,
    context::ApiContext,
    types::{ApiKeyResult, Error},
};
use async_graphql::{Context, Object, Result};
use db::ApiKey;

#[derive(Default)]
pub struct ApiKeyQuery;

#[Object]
impl ApiKeyQuery {
    /// Lists active API keys owned by the current registered user.
    async fn api_keys(&self, ctx: &Context<'_>) -> Result<Vec<ApiKeyResult>, Error> {
        let user_id = interactive_user_id(ctx)?;
        let pool = &ctx.data::<ApiContext>().unwrap().pool;
        Ok(ApiKey::list_active(pool, user_id)
            .await?
            .into_iter()
            .map(ApiKeyResult::from)
            .collect())
    }
}
