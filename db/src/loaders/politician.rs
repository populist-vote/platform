use async_graphql::async_trait::async_trait;
use async_graphql::dataloader::Loader;
use async_graphql::futures_util::TryStreamExt;
use async_graphql::FieldError;
use itertools::Itertools;

use sqlx::PgPool;
use std::collections::HashMap;

use crate::Politician;

pub struct PoliticianLoader(PgPool);

impl PoliticianLoader {
    pub fn new(pool: PgPool) -> Self {
        Self(pool.to_owned())
    }
}

#[async_trait]
impl Loader<uuid::Uuid> for PoliticianLoader {
    type Value = Politician;
    type Error = FieldError;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM politician WHERE id IN ({})"#,
            keys.iter().map(|k| format!("'{}'", k)).join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|politician: Politician| (politician.id, politician))
            .try_collect()
            .await?;

        Ok(cache)
    }
}

#[async_trait]
impl Loader<String> for PoliticianLoader {
    type Value = Politician;
    type Error = FieldError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM politician WHERE slug IN ({})"#,
            keys.iter().map(|k| format!("'{}'", k)).join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|politician: Politician| (politician.slug.clone(), politician))
            .try_collect()
            .await?;

        Ok(cache)
    }
}
