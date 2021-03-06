use async_graphql::{Context, FieldResult, Object};
use db::{Race, RaceSearch};

use crate::{context::ApiContext, types::RaceResult};

#[derive(Default)]
pub struct RaceQuery;

#[Object]
impl RaceQuery {
    async fn races(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by race title or state")] search: Option<RaceSearch>,
    ) -> FieldResult<Vec<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Race::search(&db_pool, &search.unwrap_or_default()).await?;
        let results = records.into_iter().map(RaceResult::from).collect();

        Ok(results)
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
