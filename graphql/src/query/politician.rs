use async_graphql::dataloader::*;
use async_graphql::{Context, FieldResult, Object};
use db::{Politician, PoliticianSearch};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;

use crate::relay;
use crate::types::PoliticianResult;

// struct PoliticianLoader {
//     pool: sqlx::Pool<Postgres>,
// }

// #[async_trait::async_trait]
// impl Loader<u64> for PoliticianLoader {
//     type Value = Politician;
//     type Error = Arc<sqlx::Error>;

//     async fn load(&self, keys: &[u64]) -> Result<HashMap<u64, Self::Value>, Self::Error> {
//         let query = format!("SELECT name FROM user WHERE id IN ({})", keys.iter().join(","));
//         Ok(sqlx::query_as(query)
//             .fetch(&self.pool)
//             .map_ok(|name: String| name)
//             .map_err(Arc::new)
//             .try_collect().await?)
//     }
// }

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
