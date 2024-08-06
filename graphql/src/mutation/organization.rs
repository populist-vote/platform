use crate::{
    context::ApiContext,
    guard::{OrganizationGuard, StaffOnly},
    is_admin,
    types::{Error, OrganizationResult},
    upload_to_s3, File,
};
use async_graphql::*;
use db::{
    CreateOrConnectIssueTagInput, IssueTag, IssueTagIdentifier, Organization, OrganizationRoleType,
    UpdateOrganizationInput,
};
use sqlx::{Pool, Postgres};
use std::io::Read;
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
            let new_issue_tag = IssueTag::upsert(db_pool, &input).await?;
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
    #[graphql(
        guard = "OrganizationGuard::new(&input.id.into(), &OrganizationRoleType::ReadOnly)",
        visible = "is_admin"
    )]
    async fn update_organization(
        &self,
        ctx: &Context<'_>,
        input: UpdateOrganizationInput,
    ) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Organization::update(&db_pool, &input).await?;

        if input.issue_tags.is_some() {
            handle_nested_issue_tags(&db_pool, new_record.id, input.issue_tags.unwrap()).await?;
        }

        Ok(OrganizationResult::from(new_record))
    }

    #[graphql(
        guard = "OrganizationGuard::new(&id.clone().into(), &OrganizationRoleType::ReadOnly)",
        visible = "is_admin"
    )]

    async fn upload_organization_thumbnail(
        &self,
        ctx: &Context<'_>,
        id: ID,
        file: Upload,
    ) -> Result<String> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let upload = file.value(ctx).unwrap();
        let mut content = Vec::new();
        let slug =
            db::Organization::find_by_id(&db_pool, uuid::Uuid::parse_str(id.as_str()).unwrap())
                .await?
                .slug;
        let filename = format!("{}-400", slug);
        let mimetype = upload.content_type.clone();

        upload.into_read().read_to_end(&mut content).unwrap();
        let file_info = File {
            id: ID::from(uuid::Uuid::new_v4()),
            filename,
            content,
            mimetype,
        };
        let url = upload_to_s3(file_info, "web-assets/organization-thumbnails".to_string()).await?;
        // Append last modified date because s3 path will remain the same and we want browser to cache, but refresh the image
        let url = format!("{}{}{}", url, "?lastmod=", chrono::Utc::now().timestamp());

        let result = sqlx::query_as!(
            Organization,
            r#"
            UPDATE organization
            SET assets = jsonb_set(jsonb_set(assets, '{thumbnailImage160}', $1::jsonb, true), '{thumbnailImage400}', $1::jsonb, true)
            WHERE id = $2
            RETURNING *
            "#,
            serde_json::json!(url),
            uuid::Uuid::parse_str(&id).unwrap()
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(result.assets["thumbnailImage400"]
            .as_str()
            .unwrap()
            .to_string())
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
