use async_graphql::*;
use db::{CreateIssueTagInput, IssueTag, UpdateIssueTagInput};
use sqlx::{Pool, Postgres};

use crate::{mutation::StaffOnly, types::IssueTagResult};

#[derive(Default)]
pub struct IssueTagMutation;

#[derive(SimpleObject)]
struct DeleteIssueTagResult {
    id: String,
}

#[Object]
impl IssueTagMutation {
    #[graphql(guard = "StaffOnly")]
    async fn create_issue_tag(
        &self,
        ctx: &Context<'_>,
        input: CreateIssueTagInput,
    ) -> Result<IssueTagResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = IssueTag::create(db_pool, &input).await?;
        Ok(IssueTagResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly")]
    async fn update_issue_tag(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateIssueTagInput,
    ) -> Result<IssueTagResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let updated_record = IssueTag::update(db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(IssueTagResult::from(updated_record))
    }

    #[graphql(guard = "StaffOnly")]
    async fn delete_issue_tag(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeleteIssueTagResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        IssueTag::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteIssueTagResult { id })
    }
}
