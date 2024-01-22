use async_graphql::{Context, FieldResult, Object};
use db::{Office, OfficeFilter};

use crate::{context::ApiContext, relay, types::OfficeResult};

#[derive(Default)]
pub struct OfficeQuery;

#[Object]
impl OfficeQuery {
    async fn offices(
        &self,
        ctx: &Context<'_>,
        filter: Option<OfficeFilter>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<OfficeResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Office::filter(&db_pool, &filter.unwrap_or_default()).await?;
        let results = records.into_iter().map(OfficeResult::from);

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    async fn office_by_id(&self, ctx: &Context<'_>, id: String) -> FieldResult<OfficeResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Office::find_by_id(&db_pool, uuid::Uuid::parse_str(&id).unwrap()).await?;

        Ok(record.into())
    }

    async fn office_by_slug(&self, ctx: &Context<'_>, slug: String) -> FieldResult<OfficeResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Office::find_by_slug(&db_pool, slug).await?;

        Ok(record.into())
    }
}
