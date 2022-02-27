use async_graphql::{Context, Enum, FieldResult, InputObject, Object};
use db::models::enums::PoliticalScope;
use db::{Politician, PoliticianSearch};

use crate::context::ApiContext;
use crate::relay;
use crate::types::PoliticianResult;

#[derive(Default)]
pub struct PoliticianQuery;

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Chambers {
    AllChambers,
    House,
    Senate,
}

#[derive(InputObject)]
pub struct PoliticianFilter {
    political_scope: Option<PoliticalScope>,
    chambers: Option<Chambers>,
}

impl Default for PoliticianFilter {
    fn default() -> Self {
        Self {
            political_scope: None,
            chambers: None,
        }
    }
}

#[Object(cache_control(max_age = 60))]
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
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Politician::find_by_slug(&db_pool, slug).await?;

        Ok(record.into())
    }

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
                        PoliticalScope::Federal => {
                            if office_type == "Congressional" {
                                return true;
                            } else {
                                return false;
                            };
                        }
                        PoliticalScope::State => {
                            if office_type == "State Legislative"
                                || office_type == "State Gubernatorial"
                            {
                                return true;
                            } else {
                                return false;
                            };
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
                        Chambers::AllChambers => true,
                        Chambers::House => {
                            if office_title == "Representative" {
                                return true;
                            } else {
                                return false;
                            };
                        }
                        Chambers::Senate => {
                            if office_title == "Senator" {
                                return true;
                            } else {
                                return false;
                            };
                        }
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
