use async_graphql::{Context, FieldResult, Object};
use db::{Race, RaceSearch};
use sqlx::{Pool, Postgres};

use crate::types::RaceResult;

#[derive(Default)]
pub struct RaceQuery;

#[Object]
impl RaceQuery {
    async fn races(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by race title or state")] search: Option<RaceSearch>,
    ) -> FieldResult<Vec<RaceResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Race::search(pool, &search.unwrap_or_default()).await?;
        let results = records.into_iter().map(RaceResult::from).collect();

        Ok(results)
    }

    async fn race_by_id(&self, ctx: &Context<'_>, id: String) -> FieldResult<RaceResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let record = Race::find_by_id(pool, uuid::Uuid::parse_str(&id).unwrap()).await?;

        Ok(record.into())
    }

    async fn race_by_slug(&self, ctx: &Context<'_>, slug: String) -> FieldResult<RaceResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let record = Race::find_by_slug(pool, slug).await?;

        Ok(record.into())
    }
}
