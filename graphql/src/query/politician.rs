use crate::{context::ApiContext, relay, types::PoliticianResult};
use async_graphql::{Context, Enum, InputObject, Object, Result};
use db::models::enums::PoliticalScope;
use db::{Politician, PoliticianSearch};

#[derive(Default)]
pub struct PoliticianQuery;

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Chambers {
    All,
    House,
    Senate,
}

#[derive(Default, InputObject)]
pub struct PoliticianFilter {
    political_scope: Option<PoliticalScope>,
    chambers: Option<Chambers>,
}

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
        #[graphql(desc = "Search by homeState or lastName")] search: Option<PoliticianSearch>,
        #[graphql(desc = "Filter by politicalScope or chambers")] filter: Option<PoliticianFilter>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<PoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Politician::search(&db_pool, &search.unwrap_or_default()).await?;
        let filter = filter.unwrap_or_default();
        let results: Vec<PoliticianResult> = records
            .into_iter()
            .filter(|p| {
                let office_type = &p.votesmart_candidate_bio["office"]["type"];

                if let Some(political_scope) = filter.political_scope {
                    match political_scope {
                        PoliticalScope::Federal => office_type == "Congressional",
                        PoliticalScope::State => {
                            office_type == "State Legislative"
                                || office_type == "State Gubernatorial"
                        }
                        _ => true,
                    }
                } else {
                    true
                }
            })
            .filter(|p| {
                let office_title = &p.votesmart_candidate_bio["office"]["title"];
                if let Some(chambers) = filter.chambers {
                    match chambers {
                        Chambers::All => true,
                        Chambers::House => office_title == "Representative",
                        Chambers::Senate => office_title == "Senator",
                    }
                } else {
                    true
                }
            })
            .map(PoliticianResult::from)
            .collect();

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }
}
