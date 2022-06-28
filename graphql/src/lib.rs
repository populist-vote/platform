pub mod context;
pub mod guard;
pub mod mutation;
pub mod query;
pub mod relay;
pub mod subscription;
pub mod types;

use crate::mutation::Mutation;
use crate::query::Query;

// use crate::subscription::Subscription;
use crate::types::Error;
use async_graphql::{Context, EmptySubscription, Schema, SchemaBuilder, ID};

use auth::Claims;
use dotenv::dotenv;
use http::header::HeaderName;
use http::HeaderMap;
use jsonwebtoken::TokenData;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use tracing::info;
use url::Url;

pub type PopulistSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn new_schema() -> SchemaBuilder<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
}

pub struct File {
    pub id: ID,
    pub filename: String,
    pub content: Vec<u8>,
    pub mimetype: Option<String>,
}

pub async fn upload_to_s3(file: File, directory: String) -> Result<Url, Error> {
    info!("Uploading file to s3");
    dotenv().ok();
    let accesss_key = std::env::var("AWS_ACCESS_KEY")?;
    let secret_key = std::env::var("AWS_SECRET_KEY")?;

    let bucket_name = "populist-platform";
    let region = "us-east-2".parse()?;
    let credentials = Credentials::new(
        Some(&accesss_key.to_owned()),
        Some(&secret_key.to_owned()),
        None,
        None,
        None,
    )?;
    let bucket = Bucket::new(bucket_name, region, credentials)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        "multipart/form-data".parse().unwrap(),
    );

    let path = format!("{}/{}", directory, &file.filename);

    bucket
        .put_object_with_content_type(
            format!("{}/{}", directory, &file.filename),
            &file.content,
            &file.mimetype.unwrap_or_else(|| "".to_string()),
        )
        .await?;

    let bucket_base_url = std::env::var("S3_BUCKET_BASE_URL").expect("S3_BUCKET_BASE_URL not set");
    let image_url = Url::parse(format!("{}/{}", bucket_base_url, path).as_str()).unwrap();

    Ok(image_url)
}

pub fn is_admin(ctx: &Context<'_>) -> bool {
    if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<Claims>>>() {
        matches!(
            token_data.claims.role,
            db::Role::STAFF | db::Role::SUPERUSER
        )
    } else {
        false
    }
}
