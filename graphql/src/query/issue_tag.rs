use async_graphql::{Context, FieldResult, Object};
use db::{IssueTag, IssueTagSearch};
use sqlx::{Pool, Postgres};

use crate::types::IssueTagResult;

#[derive(Default)]
pub struct IssueTagQuery;

#[Object]
impl IssueTagQuery {
    async fn issue_tag_by_slug(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search issue tag by slug")] slug: String,
    ) -> FieldResult<IssueTagResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let record = IssueTag::find_by_slug(pool, slug).await?;
        let result = IssueTagResult::from(record);
        Ok(result)
    }

    async fn all_issue_tags(&self, ctx: &Context<'_>) -> FieldResult<Vec<IssueTagResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = IssueTag::index(pool).await?;
        let results = records
            .into_iter()
            .map(|r| IssueTagResult::from(r))
            .collect();
        Ok(results)
    }

    async fn issue_tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by issue tag name")] search: IssueTagSearch,
    ) -> FieldResult<Vec<IssueTagResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = IssueTag::search(pool, &search).await?;
        let results = records
            .into_iter()
            .map(|r| IssueTagResult::from(r))
            .collect();
        Ok(results)
    }
}
