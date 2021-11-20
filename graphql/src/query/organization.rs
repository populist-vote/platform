use async_graphql::{Context, FieldResult, Object};
use db::{Organization, OrganizationSearch};
use sqlx::{Pool, Postgres};

use crate::types::OrganizationResult;

#[derive(Default)]
pub struct OrganizationQuery;

#[Object]
impl OrganizationQuery {
    async fn all_organizations(&self, ctx: &Context<'_>) -> FieldResult<Vec<OrganizationResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Organization::index(pool).await?;
        let results = records
            .into_iter()
            .map(OrganizationResult::from)
            .collect();
        Ok(results)
    }

    async fn organizations(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by organization name")] search: OrganizationSearch,
    ) -> FieldResult<Vec<OrganizationResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Organization::search(pool, &search).await?;
        let results = records
            .into_iter()
            .map(OrganizationResult::from)
            .collect();
        Ok(results)
    }
}
