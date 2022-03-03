use async_graphql::async_trait::async_trait;
use async_graphql::dataloader::Loader;
use async_graphql::futures_util::TryStreamExt;
use async_graphql::FieldError;
use itertools::Itertools;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::Organization;
pub struct OrganizationLoader(PgPool);

impl OrganizationLoader {
    pub fn new(pool: PgPool) -> Self {
        Self(pool.to_owned())
    }
}

// Currently being used for loading via Votesmart sig ids, but should also implement for org ids
#[async_trait]
impl Loader<i32> for OrganizationLoader {
    type Value = Organization;
    type Error = FieldError;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM organization WHERE votesmart_sig_id IN ({})"#,
            keys.iter().join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|org: Organization| (org.votesmart_sig_id.unwrap(), org))
            .try_collect()
            .await?;

        Ok(cache)
    }
}
