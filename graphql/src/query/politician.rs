use crate::{context::ApiContext, relay, types::PoliticianResult};
use async_graphql::{Context, Object, Result};
use db::{Politician, PoliticianFilter};

#[derive(Default)]
pub struct PoliticianQuery;

#[allow(clippy::too_many_arguments)]
#[Object]
impl PoliticianQuery {
    async fn politician_by_slug(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> Result<PoliticianResult> {
        let cached_politician = ctx
            .data::<ApiContext>()?
            .loaders
            .politician_loader
            .load_one(slug.clone())
            .await?;

        if let Some(politician) = cached_politician {
            Ok(politician.into())
        } else {
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = Politician::find_by_slug(&db_pool, slug).await?;
            Ok(record.into())
        }
    }

    #[allow(clippy::needless_collect)]
    async fn politicians(
        &self,
        ctx: &Context<'_>,
        filter: Option<PoliticianFilter>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<PoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Politician::filter(&db_pool, &filter.unwrap_or_default()).await?;
        let results: Vec<PoliticianResult> =
            records.into_iter().map(PoliticianResult::from).collect();

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }
}
