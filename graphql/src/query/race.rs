use async_graphql::{Context, FieldResult, Object};
use db::{Race, RaceFilter};

use crate::{context::ApiContext, relay, types::RaceResult};

#[derive(Default)]
pub struct RaceQuery;

#[Object]
impl RaceQuery {
    async fn races(
        &self,
        ctx: &Context<'_>,
        filter: Option<RaceFilter>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Race::filter(&db_pool, filter.unwrap_or_default()).await?;
        let results: Vec<RaceResult> = records.into_iter().map(RaceResult::from).collect();

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            20,
        )
        .await
    }

    async fn race_by_id(&self, ctx: &Context<'_>, id: String) -> FieldResult<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Race::find_by_id(&db_pool, uuid::Uuid::parse_str(&id).unwrap()).await?;

        Ok(record.into())
    }

    async fn race_by_slug(&self, ctx: &Context<'_>, slug: String) -> FieldResult<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Race::find_by_slug(&db_pool, slug).await?;

        Ok(record.into())
    }
}
