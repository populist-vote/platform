use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::IssueTagResult};
use async_graphql::*;
use db::{CreateIssueTagInput, IssueTag, UpdateIssueTagInput};

#[derive(Default)]
pub struct IssueTagMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteIssueTagResult {
    id: String,
}

#[Object]
impl IssueTagMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn create_issue_tag(
        &self,
        ctx: &Context<'_>,
        input: CreateIssueTagInput,
    ) -> Result<IssueTagResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = IssueTag::create(&db_pool, &input).await?;
        Ok(IssueTagResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn update_issue_tag(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateIssueTagInput,
    ) -> Result<IssueTagResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_record =
            IssueTag::update(&db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(IssueTagResult::from(updated_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_issue_tag(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeleteIssueTagResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        IssueTag::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteIssueTagResult { id })
    }
}
