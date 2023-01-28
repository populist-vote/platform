use async_graphql::{Context, Object, Result};
use db::{Organization, OrganizationSearch};

use crate::context::ApiContext;
use crate::relay;
use crate::types::OrganizationResult;

#[derive(Default)]
pub struct OrganizationQuery;

#[Object]
impl OrganizationQuery {
    async fn organizations(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by organization name")] search: Option<OrganizationSearch>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Organization::search(&db_pool, &search.unwrap_or_default()).await?;
        let results = records.into_iter().map(OrganizationResult::from);

        relay::query(results, relay::Params::new(after, before, first, last), 10).await
    }

    async fn organization_by_slug(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Organization::find_by_slug(&db_pool, slug).await?;

        Ok(record.into())
    }

    async fn organization_by_id(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Organization::find_by_id(&db_pool, uuid::Uuid::parse_str(&id)?).await?;

        Ok(record.into())
    }
}
