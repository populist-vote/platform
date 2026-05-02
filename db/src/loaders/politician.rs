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

impl Loader<OfficeId> for PoliticianLoader {
    /// All politicians holding this office (`politician.office_id`); may be empty or many rows per office.
    type Value = Vec<Politician>;
    type Error = FieldError;

    async fn load(&self, keys: &[OfficeId]) -> Result<HashMap<OfficeId, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let query = format!(
            r#"SELECT * FROM politician WHERE office_id IN ({}) ORDER BY last_name ASC, first_name ASC, id ASC"#,
            keys.iter().map(|k| format!("'{}'", k.0)).join(",")
        );

        let rows: Vec<Politician> = sqlx::query_as(&query).fetch_all(&self.0).await?;

        let mut cache: HashMap<OfficeId, Vec<Politician>> =
            keys.iter().cloned().map(|k| (k, Vec::new())).collect();

        for politician in rows {
            if let Some(oid) = politician.office_id {
                let key = OfficeId(oid);
                if let Some(vec) = cache.get_mut(&key) {
                    vec.push(politician);
                }
            }
        }

        Ok(cache)
    }
}
