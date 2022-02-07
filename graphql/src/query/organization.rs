use async_graphql::{Context, FieldResult, Object};
use db::{Organization, OrganizationSearch};
use sqlx::{Pool, Postgres};

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
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Organization::search(pool, &search.unwrap_or_default()).await?;
        let results = records.into_iter().map(OrganizationResult::from);

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    async fn organization_by_slug(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> FieldResult<OrganizationResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let record = Organization::find_by_slug(pool, slug).await?;

        Ok(record.into())
    }
}
