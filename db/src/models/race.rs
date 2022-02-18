use async_graphql::{InputObject, ID};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use slugify::slugify;
use sqlx::PgPool;

use crate::DateTime;

use super::enums::State;

#[derive(sqlx::FromRow, Debug, Clone)]

pub struct Race {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub office_position: String,
    pub office_id: uuid::Uuid,
    pub state: Option<State>,
    pub description: Option<String>,
    pub ballotpedia_link: Option<String>,
    pub early_voting_begins_date: Option<NaiveDate>,
    pub election_date: Option<NaiveDate>,
    pub official_website: Option<String>,
    pub election_id: Option<uuid::Uuid>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateRaceInput {
    pub slug: Option<String>,
    pub title: String,
    pub office_position: String,
    pub office_id: ID,
    pub state: Option<State>,
    pub description: Option<String>,
    pub ballotpedia_link: Option<String>,
    pub early_voting_begins_date: Option<NaiveDate>,
    pub election_date: Option<NaiveDate>,
    pub official_website: Option<String>,
    pub election_id: Option<ID>,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdateRaceInput {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub office_position: Option<String>,
    pub office_id: Option<ID>,
    pub description: Option<String>,
    pub ballotpedia_link: Option<String>,
    pub early_voting_begins_date: Option<NaiveDate>,
    pub election_date: Option<NaiveDate>,
    pub official_website: Option<String>,
    pub state: Option<State>,
    pub election_id: Option<ID>,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct RaceSearch {
    query: Option<String>,
    state: Option<State>,
}

impl Default for RaceSearch {
    fn default() -> Self {
        Self {
            query: None,
            state: None,
        }
    }
}

impl Race {
    pub async fn create(db_pool: &PgPool, input: &CreateRaceInput) -> Result<Self, sqlx::Error> {
        let slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.title),
        };

        let record = sqlx::query_as!(
            Race,
            r#"
                INSERT INTO race (slug, title, office_position, office_id, state,  description, ballotpedia_link, early_voting_begins_date, election_date, official_website, election_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING id, slug, title, office_position, office_id, state AS "state:State", description, ballotpedia_link, early_voting_begins_date, election_date, official_website, election_id, created_at, updated_at
            "#,
            slug,
            input.title,
            input.office_position,
            uuid::Uuid::parse_str(input.office_id.to_owned().as_str()).unwrap(),
            input.state as Option<State>,
            input.description,
            input.ballotpedia_link,
            input.early_voting_begins_date,
            input.election_date,
            input.official_website,
            uuid::Uuid::parse_str(input.election_id.to_owned().unwrap_or_default().as_str()).unwrap(),
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateRaceInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Race,
            r#"
                UPDATE race
                SET slug = COALESCE($2, slug), 
                    title = COALESCE($3, title), 
                    office_position = COALESCE($4, office_position),
                    office_id = COALESCE($5, office_id),
                    state = COALESCE($6, state),
                    description = COALESCE($7, description),
                    ballotpedia_link = COALESCE($8, ballotpedia_link),
                    early_voting_begins_date = COALESCE($9, early_voting_begins_date),
                    election_date = COALESCE($10, election_date),
                    official_website = COALESCE($11, official_website),
                    election_id = COALESCE($12, election_id)
                WHERE id = $1
                RETURNING id, slug, title, office_position, office_id, state AS "state:State", description, ballotpedia_link, early_voting_begins_date, election_date, official_website, election_id, created_at, updated_at
            "#,
            id,
            input.slug,
            input.title,
            input.office_position,
            uuid::Uuid::parse_str(input.office_id.to_owned().unwrap_or_default().as_str()).unwrap(),
            input.state as Option<State>,
            input.description,
            input.ballotpedia_link,
            input.early_voting_begins_date,
            input.election_date,
            input.official_website,
            uuid::Uuid::parse_str(input.election_id.to_owned().unwrap_or_default().as_str()).unwrap(),
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
                SELECT id, slug, title, office_position, office_id, state AS "state:State", description, ballotpedia_link, early_voting_begins_date, election_date, official_website, election_id, created_at, updated_at FROM race
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
                SELECT id, slug, title, office_position, office_id, state AS "state:State", description, ballotpedia_link, early_voting_begins_date, election_date, official_website, election_id, created_at, updated_at FROM race
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
                SELECT id, slug, title, office_position, office_id, state AS "state:State", description, ballotpedia_link, early_voting_begins_date, election_date, official_website, election_id, created_at, updated_at FROM race
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
