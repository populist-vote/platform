use async_graphql::*;
use db::{
    CreateOrConnectIssueTagInput, CreatePoliticianInput, IssueTag, Politician,
    UpdatePoliticianInput,
};
use sqlx::{Pool, Postgres};

use crate::{
    types::{Error, PoliticianResult},
    upload_to_s3, File,
};

use std::io::Read;
#[derive(Default)]
pub struct PoliticianMutation;

#[derive(SimpleObject)]
struct DeletePoliticianResult {
    id: String,
}

// Create or connect issue tags with relation to new or updated politician
async fn handle_nested_issue_tags(
    db_pool: &Pool<Postgres>,
    associated_record_id: uuid::Uuid,
    issue_tags_input: CreateOrConnectIssueTagInput,
) -> Result<(), Error> {
    if issue_tags_input.create.is_some() {
        for input in issue_tags_input.create.unwrap() {
            let new_issue_tag = IssueTag::create(db_pool, &input).await?;
            Politician::connect_issue_tag(db_pool, associated_record_id, new_issue_tag.id).await?;
        }
    }
    if issue_tags_input.connect.is_some() {
        for issue_tag_id in issue_tags_input.connect.unwrap() {
            // figure out how to accept slugs and IDs here, that'd be great
            Politician::connect_issue_tag(
                db_pool,
                associated_record_id,
                uuid::Uuid::parse_str(&issue_tag_id)?,
            )
            .await?;
        }
    }
    Ok(())
}

#[Object]
impl PoliticianMutation {
    async fn create_politician(
        &self,
        ctx: &Context<'_>,
        input: CreatePoliticianInput,
    ) -> Result<PoliticianResult, Error> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = Politician::create(db_pool, &input).await?;

        handle_nested_issue_tags(db_pool, new_record.id, input.issue_tags.unwrap()).await?;

        Ok(PoliticianResult::from(new_record))
    }

    async fn update_politician(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdatePoliticianInput,
    ) -> Result<PoliticianResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let updated_record =
            Politician::update(db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;

        handle_nested_issue_tags(db_pool, updated_record.id, input.issue_tags.unwrap()).await?;
        
        Ok(PoliticianResult::from(updated_record))
    }

    // TODO make this generic and accept an associated model e.g Politician
    async fn upload_politician_thumbnail(
        &self,
        ctx: &Context<'_>,
        file: Upload,
    ) -> Result<u16, Error> {
        let upload = file.value(ctx).unwrap();
        let mut content = Vec::new();
        let filename = upload.filename.clone();
        let mimetype = upload.content_type.clone();

        upload.into_read().read_to_end(&mut content).unwrap();
        let file_info = File {
            id: ID::from(uuid::Uuid::new_v4()),
            filename,
            content,
            mimetype,
        };
        Ok(upload_to_s3(file_info).await?)
    }

    async fn delete_politician(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeletePoliticianResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        Politician::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeletePoliticianResult { id })
    }
}
