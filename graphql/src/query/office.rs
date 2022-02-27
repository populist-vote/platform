use async_graphql::{Context, FieldResult, Object};
use db::{Office, OfficeSearch};

use crate::{context::ApiContext, types::OfficeResult};

#[derive(Default)]
pub struct OfficeQuery;

#[Object]
impl OfficeQuery {
    async fn offices(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by office title or state")] search: Option<OfficeSearch>,
    ) -> FieldResult<Vec<OfficeResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Office::search(&db_pool, &search.unwrap_or_default()).await?;
        let results = records.into_iter().map(OfficeResult::from).collect();

        Ok(results)
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
