use async_graphql::{Context, FieldResult, Object};
use db::{Politician, PoliticianSearch};
use sqlx::{Pool, Postgres};

use crate::relay;
use crate::types::PoliticianResult;

#[derive(Default)]
pub struct PoliticianQuery;

#[Object]
impl PoliticianQuery {
    async fn politician_by_id(
        &self,
        _ctx: &Context<'_>,
        _id: String,
    ) -> FieldResult<PoliticianResult> {
        // Look up politician by id in the database
        todo!()
    }

    async fn politician_by_slug(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> FieldResult<PoliticianResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let record = Politician::find_by_slug(pool, slug).await?;

        Ok(record.into())
    }

    async fn politicians(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by homeState or lastName")] search: Option<PoliticianSearch>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<PoliticianResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Politician::search(pool, &search.unwrap_or_default()).await?;
        let results = records.into_iter().map(PoliticianResult::from);

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }
}
