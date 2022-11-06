use async_graphql::async_trait::async_trait;
use async_graphql::dataloader::Loader;
use async_graphql::futures_util::TryStreamExt;
use async_graphql::FieldError;
use itertools::Itertools;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::IssueTag;
pub struct IssueTagLoader(PgPool);

impl IssueTagLoader {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

// Load issue tags by id
#[async_trait]
impl Loader<uuid::Uuid> for IssueTagLoader {
    type Value = IssueTag;
    type Error = FieldError;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM issue_tag WHERE id IN ({})"#,
            keys.iter().join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|tag: IssueTag| (tag.id, tag))
            .try_collect()
            .await?;

        Ok(cache)
    }
}
