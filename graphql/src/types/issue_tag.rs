use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::models::issue_tag::IssueTag;

use crate::context::ApiContext;

use super::{BallotMeasureResult, BillResult, OrganizationResult, PoliticianResult};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct IssueTagResult {
    id: ID,
    slug: String,
    name: String,
    description: Option<String>,
}

#[ComplexObject]
impl IssueTagResult {
    async fn politicians(&self, _ctx: &Context<'_>) -> FieldResult<Vec<PoliticianResult>> {
        todo!()
    }

    async fn organizations(&self, ctx: &Context<'_>) -> FieldResult<Vec<OrganizationResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = IssueTag::organizations(&db_pool, uuid::Uuid::parse_str(&self.id)?).await?;
        let results = records.into_iter().map(OrganizationResult::from).collect();
        Ok(results)
    }

    async fn bills(&self, _ctx: &Context<'_>) -> FieldResult<Vec<BillResult>> {
        todo!()
    }

    async fn ballot_measures(&self, _ctx: &Context<'_>) -> FieldResult<Vec<BallotMeasureResult>> {
        todo!()
    }
}

impl From<IssueTag> for IssueTagResult {
    fn from(it: IssueTag) -> Self {
        Self {
            id: ID::from(it.id),
            slug: it.slug,
            name: it.name,
            description: it.description,
        }
    }
}
