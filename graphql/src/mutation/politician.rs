use async_graphql::*;
use db::{CreatePoliticianInput, Politician, UpdatePoliticianInput};
use sqlx::{Pool, Postgres};

use crate::{
    types::{PoliticianResult},
    upload_to_s3, File,
};

use std::io::Read;
#[derive(Default)]
pub struct PoliticianMutation;

#[derive(SimpleObject)]
struct DeletePoliticianResult {
    id: String,
}

#[Object]
impl PoliticianMutation {
    async fn create_politician(
        &self,
        ctx: &Context<'_>,
        input: CreatePoliticianInput,
    ) -> Result<PoliticianResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = Politician::create(db_pool, &input).await?;
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
