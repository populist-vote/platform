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
            r#"SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", office_id, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, votesmart_candidate_ratings, legiscan_people_id, upcoming_race_id, created_at, updated_at FROM politician WHERE id IN ({})"#,
            keys.iter().join(",")
        );

        let cache = sqlx::query_as(&query)
            .fetch(&self.0)
            .map_ok(|politician: Politician| (politician.id, politician))
            .try_collect()
            .await?;

        Ok(cache)
    }
}
