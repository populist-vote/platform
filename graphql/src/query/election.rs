use async_graphql::{Context, FieldResult, Object};
use db::{Election, ElectionSearchInput};
use sqlx::{Pool, Postgres};

use crate::types::ElectionResult;

#[derive(Default)]
pub struct ElectionQuery;

#[Object]
impl ElectionQuery {
    async fn all_elections(&self, ctx: &Context<'_>) -> FieldResult<Vec<ElectionResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Election::index(pool).await?;
        let results = records
            .into_iter()
            .map(|r| ElectionResult::from(r))
            .collect();
        Ok(results)
    }

    async fn elections(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by slug or title")] search: ElectionSearchInput,
    ) -> FieldResult<Vec<ElectionResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Election::search(pool, &search).await?;
        let results = records
            .into_iter()
            .map(|r| ElectionResult::from(r))
            .collect();
        Ok(results)
    }

    async fn upcoming_elections(&self, _ctx: &Context<'_>) -> FieldResult<Vec<ElectionResult>> {
        todo!();
    }

    async fn election_by_id(&self, _ctx: &Context<'_>, _id: String) -> FieldResult<ElectionResult> {
        // Look up election by id in the database
        todo!()
    }
}
