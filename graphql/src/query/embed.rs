use async_graphql::{Context, Object, Result, ID};
use auth::Claims;
use db::Embed;
use jsonwebtoken::TokenData;

use crate::{context::ApiContext, guard::OrganizationGuard, is_admin, types::EmbedResult};

#[derive(Default)]
pub struct EmbedQuery;

#[Object]
impl EmbedQuery {
    #[graphql(
        guard = "OrganizationGuard::new(&organization_id)",
        visible = "is_admin"
    )]
    async fn embeds_by_organization(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
    ) -> Result<Vec<EmbedResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records =
            Embed::find_by_organization_id(&db_pool, uuid::Uuid::parse_str(&organization_id)?)
                .await?;
        let results = records.into_iter().map(EmbedResult::from).collect();
        Ok(results)
    }
    #[graphql(visible = "is_admin")]
    async fn embed_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<EmbedResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Embed::find_by_id(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<Claims>>>() {
            if token_data.claims.organization_id.unwrap_or_default() != record.organization_id {
                return Err("Unauthorized".into());
            }
        }
        Ok(record.into())
    }
}
