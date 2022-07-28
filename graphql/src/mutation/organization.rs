use crate::{
    context::ApiContext,
    guard::StaffOnly,
    is_admin,
    types::{Error, OrganizationResult},
};
use async_graphql::*;
use db::{
    CreateOrConnectIssueTagInput, CreateOrganizationInput, IssueTag, IssueTagIdentifier,
    Organization, UpdateOrganizationInput,
};
use sqlx::{Pool, Postgres};
use std::str::FromStr;
#[derive(Default)]
pub struct OrganizationMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteOrganizationResult {
    id: String,
}

pub async fn handle_nested_issue_tags(
    db_pool: &Pool<Postgres>,
    associated_record_id: uuid::Uuid,
    issue_tags_input: CreateOrConnectIssueTagInput,
) -> Result<(), Error> {
    if issue_tags_input.create.is_some() {
        for input in issue_tags_input.create.unwrap() {
            let new_issue_tag = IssueTag::create(db_pool, &input).await?;
            Organization::connect_issue_tag(
                db_pool,
                associated_record_id,
                IssueTagIdentifier::Uuid(new_issue_tag.id),
            )
            .await?;
        }
    }
    if issue_tags_input.connect.is_some() {
        for issue_tag_identifier in issue_tags_input.connect.unwrap() {
            match uuid::Uuid::from_str(issue_tag_identifier.as_str()) {
                Ok(issue_tag_id) => {
                    Organization::connect_issue_tag(
                        db_pool,
                        associated_record_id,
                        IssueTagIdentifier::Uuid(issue_tag_id),
                    )
                    .await?;
                }
                _ => {
                    Organization::connect_issue_tag(
                        db_pool,
                        associated_record_id,
                        IssueTagIdentifier::Slug(issue_tag_identifier),
                    )
                    .await?
                }
            }
        }
    }
    Ok(())
}

#[Object]
impl OrganizationMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn create_organization(
        &self,
        ctx: &Context<'_>,
        input: CreateOrganizationInput,
    ) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Organization::create(&db_pool, &input).await?;

        if input.issue_tags.is_some() {
            handle_nested_issue_tags(&db_pool, new_record.id, input.issue_tags.unwrap()).await?;
        }

        Ok(OrganizationResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn update_organization(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateOrganizationInput,
    ) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_record =
            Organization::update(&db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;

        if input.issue_tags.is_some() {
            handle_nested_issue_tags(&db_pool, updated_record.id, input.issue_tags.unwrap())
                .await?;
        }

        Ok(OrganizationResult::from(updated_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_organization(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeleteOrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Organization::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteOrganizationResult { id })
    }
}
