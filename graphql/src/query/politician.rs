use async_graphql::{Context, FieldResult, Object};
use db::{Politician, PoliticianSearch};
use sqlx::{Pool, Postgres};

use crate::types::PoliticianResult;

#[derive(Default)]
pub struct PoliticianQuery;

#[Object]
impl PoliticianQuery {
    async fn all_politicians(&self, ctx: &Context<'_>) -> FieldResult<Vec<PoliticianResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Politician::index(pool).await?;
        let results = records
            .into_iter()
            .map(|r| PoliticianResult::from(r))
            .collect();
        Ok(results)
    }

    async fn politicians(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by homeState or lastName")] search: PoliticianSearch,
    ) -> FieldResult<Vec<PoliticianResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Politician::search(pool, &search).await?;
        let results = records
            .into_iter()
            .map(|r| PoliticianResult::from(r))
            .collect();
        Ok(results)
    }

    async fn politician_by_id(
        &self,
        _ctx: &Context<'_>,
        _id: String,
    ) -> FieldResult<PoliticianResult> {
        // Look up politician by id in the database
        todo!()
    }
}
