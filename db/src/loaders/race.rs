use async_graphql::dataloader::Loader;
use async_graphql::futures_util::TryStreamExt;
use async_graphql::FieldError;
use itertools::Itertools;

use sqlx::PgPool;
use std::collections::HashMap;

use crate::Race;

pub struct RaceLoader(PgPool);

impl RaceLoader {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

impl Loader<uuid::Uuid> for RaceLoader {
    type Value = Race;
    type Error = FieldError;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        let query = format!(
            r#"SELECT * FROM race WHERE id IN ({})"#,
            keys.iter().map(|k| format!("'{}'", k)).join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|race: Race| (race.id, race))
            .try_collect()
            .await?;

        Ok(cache)
    }
}
