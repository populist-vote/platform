use async_graphql::{Context, Object, Result, ID};
use db::Embed;

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
}
