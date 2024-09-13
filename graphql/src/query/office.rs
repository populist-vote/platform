use async_graphql::{Context, FieldResult, Object};
use db::{models::enums::State, Office, OfficeFilter};

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
            100,
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

    async fn counties_by_state(&self, ctx: &Context<'_>, state: State) -> FieldResult<Vec<String>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        // Fetch distinct county names from the database
        let records = sqlx::query!(
            "SELECT DISTINCT county FROM office WHERE state = $1",
            state as State
        )
        .fetch_all(&db_pool)
        .await?;

        // Map the resulting records to a vector of county strings
        let counties: Vec<String> = records.into_iter().filter_map(|rec| rec.county).collect();

        Ok(counties)
    }
}
