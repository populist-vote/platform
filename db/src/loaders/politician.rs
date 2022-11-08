use async_graphql::async_trait::async_trait;
use async_graphql::dataloader::Loader;
use async_graphql::futures_util::TryStreamExt;
use async_graphql::FieldError;
use itertools::Itertools;

use sqlx::PgPool;
use std::collections::HashMap;

use crate::Politician;

pub struct PoliticianLoader(PgPool);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PoliticianId(pub uuid::Uuid);
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PoliticianSlug(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OfficeId(pub uuid::Uuid);

impl PoliticianLoader {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[async_trait]
impl Loader<PoliticianId> for PoliticianLoader {
    type Value = Politician;
    type Error = FieldError;

    async fn load(
        &self,
        keys: &[PoliticianId],
    ) -> Result<HashMap<PoliticianId, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM politician WHERE id IN ({})"#,
            keys.iter().map(|k| format!("'{}'", k.0)).join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|politician: Politician| (PoliticianId(politician.id), politician))
            .try_collect()
            .await?;

        Ok(cache)
    }
}

#[async_trait]
impl Loader<PoliticianSlug> for PoliticianLoader {
    type Value = Politician;
    type Error = FieldError;

    async fn load(
        &self,
        keys: &[PoliticianSlug],
    ) -> Result<HashMap<PoliticianSlug, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM politician WHERE slug IN ({})"#,
            keys.iter().map(|k| format!("'{}'", k.0)).join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|politician: Politician| (PoliticianSlug(politician.slug.clone()), politician))
            .try_collect()
            .await?;

        Ok(cache)
    }
}

#[async_trait]
impl Loader<OfficeId> for PoliticianLoader {
    type Value = Politician;
    type Error = FieldError;

    async fn load(&self, keys: &[OfficeId]) -> Result<HashMap<OfficeId, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM politician WHERE office_id IN ({})"#,
            keys.iter().map(|k| format!("'{}'", k.0)).join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|politician: Politician| {
                (
                    OfficeId(politician.office_id.unwrap_or_default()),
                    politician,
                )
            })
            .try_collect()
            .await?;

        Ok(cache)
    }
}
