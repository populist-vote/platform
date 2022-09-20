use super::enums::{PoliticalParty, RaceType, State};
use crate::DateTime;
use async_graphql::InputObject;
use chrono::NaiveDate;
use rand::Rng;
use serde::{Deserialize, Serialize};
use slugify::slugify;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]

pub struct Race {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub office_id: uuid::Uuid,
    pub race_type: RaceType,
    pub party: Option<PoliticalParty>,
    pub state: Option<State>,
    pub description: Option<String>,
    pub ballotpedia_link: Option<String>,
    pub early_voting_begins_date: Option<NaiveDate>,
    pub official_website: Option<String>,
    pub election_id: Option<uuid::Uuid>,
    pub winner_id: Option<uuid::Uuid>,
    pub total_votes: Option<i32>,
    pub is_special_election: bool,
    pub num_elect: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpsertRaceInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub office_id: Option<uuid::Uuid>,
    pub race_type: Option<RaceType>,
    pub party: Option<PoliticalParty>,
    pub description: Option<String>,
    pub ballotpedia_link: Option<String>,
    pub early_voting_begins_date: Option<NaiveDate>,
    pub official_website: Option<String>,
    pub state: Option<State>,
    pub election_id: Option<uuid::Uuid>,
    pub winner_id: Option<uuid::Uuid>,
    pub total_votes: Option<i32>,
    pub is_special_election: bool,
    pub num_elect: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize, InputObject)]
pub struct RaceSearch {
    query: Option<String>,
    state: Option<State>,
}

impl Race {
    pub async fn upsert(db_pool: &PgPool, input: &UpsertRaceInput) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(uuid::Uuid::new_v4);
        let mut slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.title.clone().unwrap_or_default()),
        };

        let existing_slug = sqlx::query!(
            r#"
            SELECT slug
            FROM race
            WHERE slug = $1 AND id != $2
            "#,
            input.slug,
            input.id
        )
        .fetch_optional(db_pool)
        .await?;

        let rando: i32 = { rand::thread_rng().gen() };

        if let Some(r) = existing_slug {
            slug = format!("{}-{}", r.slug, rando);
        }

        let record = sqlx::query_as!(Race,
            r#"
                INSERT INTO race (id, slug, title, office_id, race_type, party, state,  description, ballotpedia_link, early_voting_begins_date, winner_id, official_website, election_id, total_votes, is_special_election, num_elect)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT (id) DO UPDATE
                SET
                    slug = COALESCE($2, race.slug),
                    title = COALESCE($3, race.title), 
                    office_id = COALESCE($4, race.office_id),
                    race_type = COALESCE($5, race.race_type),
                    party = COALESCE($6, race.party),
                    state = COALESCE($7, race.state),
                    description = COALESCE($8, race.description),
                    ballotpedia_link = COALESCE($9, race.ballotpedia_link),
                    early_voting_begins_date = COALESCE($10, race.early_voting_begins_date),
                    winner_id = COALESCE($11, race.winner_id),
                    official_website = COALESCE($12, race.official_website),
                    election_id = COALESCE($13, race.election_id),
                    total_votes = COALESCE($14, race.total_votes),
                    is_special_election = COALESCE($15, race.is_special_election),
                    num_elect = COALESCE($16, race.num_elect)
                RETURNING id, slug, title,  office_id, race_type AS "race_type:RaceType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, winner_id, official_website, election_id, total_votes, is_special_election, num_elect, created_at, updated_at
            "#,
            id,
            slug,
            input.title,
            input.office_id,
            input.race_type as Option<RaceType>,
            input.party as Option<PoliticalParty>,
            input.state as Option<State>,
            input.description,
            input.ballotpedia_link,
            input.early_voting_begins_date,
            input.winner_id,
            input.official_website,
            input.election_id,
            input.total_votes,
            input.is_special_election,
            input.num_elect,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM race WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Race,
            r#"
                SELECT id, slug, title, office_id, race_type AS "race_type:RaceType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, winner_id, total_votes, official_website, election_id,  is_special_election, num_elect, created_at, updated_at FROM race
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_slug(db_pool: &PgPool, slug: String) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Race,
            r#"
                SELECT id, slug, title, office_id, race_type AS "race_type:RaceType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, winner_id, total_votes, official_website, election_id, is_special_election, num_elect, created_at, updated_at FROM race
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn search(db_pool: &PgPool, input: &RaceSearch) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Race,
            r#"
                SELECT id, slug, title, office_id, race_type AS "race_type:RaceType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, winner_id, total_votes, official_website, election_id, is_special_election, num_elect, created_at, updated_at FROM race
                WHERE (($1::text = '') IS NOT FALSE OR to_tsvector(concat_ws(' ', slug, title)) @@ to_tsquery($1))
                AND ($2::state IS NULL OR state = $2)
                
            "#,
            input.query,
            input.state as Option<State>,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
