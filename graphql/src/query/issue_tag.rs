use async_graphql::{Context, FieldResult, Object};
use db::{IssueTag, IssueTagSearch};

use crate::{context::ApiContext, types::IssueTagResult};

#[derive(Default)]
pub struct IssueTagQuery;

#[Object]
impl IssueTagQuery {
    async fn issue_tag_by_slug(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search issue tag by slug")] slug: String,
    ) -> FieldResult<IssueTagResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = IssueTag::find_by_slug(&db_pool, slug).await?;
        let result = IssueTagResult::from(record);
        Ok(result)
    }

    async fn all_issue_tags(&self, ctx: &Context<'_>) -> FieldResult<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = IssueTag::index(&db_pool).await?;
        let results = records.into_iter().map(IssueTagResult::from).collect();
        Ok(results)
    }

    async fn issue_tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by issue tag name")] search: IssueTagSearch,
    ) -> FieldResult<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = IssueTag::search(&db_pool, &search).await?;
        let results = records.into_iter().map(IssueTagResult::from).collect();
        Ok(results)
    }
}
