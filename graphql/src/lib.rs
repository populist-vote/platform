pub mod context;
pub mod guard;
pub mod mutation;
pub mod query;
pub mod relay;
pub mod subscription;
pub mod test;
pub mod types;

use std::{fmt, net::SocketAddr};

use crate::{mutation::Mutation, query::Query, types::Error};
use async_graphql::extensions::Tracing;
use async_graphql::{Context, Schema, SchemaBuilder, ID};
use auth::AccessTokenClaims;
use dotenv::dotenv;
use http::header::HeaderName;
use http::HeaderMap;
use jsonwebtoken::TokenData;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use subscription::Subscription;
use tracing::info;
use url::Url;

#[derive(Debug, Clone)]
// Wrapper type representing a client session ID, used to track anonymous user sessions
pub struct SessionID(String);
#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: SessionID,
    pub ip: SocketAddr,
}

impl From<String> for SessionID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for SessionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type PopulistSchema = Schema<Query, Mutation, Subscription>;

pub fn new_schema() -> SchemaBuilder<Query, Mutation, Subscription> {
    Schema::build(
        Query::default(),
        Mutation::default(),
        Subscription::default(),
    )
    .limit_depth(16)
    .limit_complexity(256)
    .limit_recursive_depth(64)
    .extension(Tracing)
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
    let bucket_name = std::env::var("AWS_S3_BUCKET")?;
    let region = match bucket_name.to_owned().as_str() {
        "populist-platform" => "us-east-2".parse().unwrap(),
        "populist-platform-staging" => "us-east-1".parse().unwrap(),
        _ => "us-east-2".parse().unwrap(),
    };
    let credentials = Credentials::new(
        Some(&accesss_key.to_owned()),
        Some(&secret_key.to_owned()),
        None,
        None,
        None,
    )?;
    let bucket = Bucket::new(&bucket_name, region, credentials)?;
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
            &file.mimetype.unwrap_or_default(),
        )
        .await
        .unwrap();
    let bucket_base_url = std::env::var("S3_BUCKET_BASE_URL").expect("S3_BUCKET_BASE_URL not set");
    let image_url = Url::parse(format!("{}/{}", bucket_base_url, path).as_str()).unwrap();

    Ok(image_url)
}

pub async fn delete_from_s3(path: String) -> Result<(), Error> {
    info!("Deleting file from s3");
    dotenv().ok();
    let accesss_key = std::env::var("AWS_ACCESS_KEY")?;
    let secret_key = std::env::var("AWS_SECRET_KEY")?;

    let bucket_name = "populist-platform";
    let region = "us-east-2".parse().unwrap();
    let credentials = Credentials::new(
        Some(&accesss_key.to_owned()),
        Some(&secret_key.to_owned()),
        None,
        None,
        None,
    )
    .unwrap();
    let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    bucket.delete_object(path).await.unwrap();
    Ok(())
}

pub fn is_admin(ctx: &Context<'_>) -> bool {
    if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<AccessTokenClaims>>>() {
        matches!(
            token_data.claims.system_role,
            db::SystemRoleType::Staff | db::SystemRoleType::Superuser
        )
    } else {
        false
    }
}
