use super::enums::{PoliticalScope, State};
use crate::DateTime;
use async_graphql::{Enum, InputObject};
use serde::{Deserialize, Serialize};
use slugify::slugify;
use sqlx::PgPool;
use strum_macros::{Display, EnumString};

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Office {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub political_scope: PoliticalScope,
    pub office_type: Option<String>,
    pub district: Option<String>,
    pub district_type: Option<District>,
    pub chamber: Option<Chamber>,
    /// Helps to determine which races to display to a user
    pub election_scope: ElectionScope,
    pub state: Option<State>,
    pub municipality: Option<String>,
    /// If a new office is introduced for redistricting or other reasons,
    /// there may not be an incumbent
    pub incumbent_id: Option<uuid::Uuid>,
    pub term_length: Option<i32>,
    pub seat: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
#[derive(
    Display,
    Default,
    Enum,
    Debug,
    Copy,
    Clone,
    Hash,
    Eq,
    PartialEq,
    EnumString,
    sqlx::Type,
    Serialize,
    Deserialize,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "election_scope", rename_all = "lowercase")]
pub enum ElectionScope {
    #[default]
    National,
    State,
    County,
    City,
    District,
}

#[derive(
    Display, Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, sqlx::Type, Serialize, Deserialize,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "district_type", rename_all = "snake_case")]
pub enum District {
    UsCongressional,
    StateSenate,
    StateHouse,
    School,
    City,
    County,
}

#[derive(
    Display, Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, sqlx::Type, Serialize, Deserialize,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "chamber", rename_all = "lowercase")]
pub enum Chamber {
    House,
    Senate,
}

#[derive(Debug, Default, Serialize, Deserialize, InputObject)]
pub struct CreateOfficeInput {
    pub slug: Option<String>,
    pub title: String,
    pub office_type: Option<String>,
    pub district: Option<String>,
    pub district_type: Option<District>,
    pub chamber: Option<Chamber>,
    pub election_scope: ElectionScope,
    pub political_scope: PoliticalScope,
    pub state: Option<State>,
    pub municipality: Option<String>,
    pub incumbent_id: Option<uuid::Uuid>,
    pub term_length: Option<i32>,
    pub seat: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdateOfficeInput {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub office_type: Option<String>,
    pub district: Option<String>,
    pub district_type: Option<District>,
    pub chamber: Option<Chamber>,
    pub election_scope: Option<ElectionScope>,
    pub political_scope: Option<PoliticalScope>,
    pub state: Option<State>,
    pub municipality: Option<String>,
    pub incumbent_id: Option<uuid::Uuid>,
    pub term_length: Option<i32>,
    pub seat: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, InputObject)]
pub struct OfficeSearch {
    query: Option<String>,
    state: Option<State>,
    political_scope: Option<PoliticalScope>,
}

impl Office {
    pub async fn create(db_pool: &PgPool, input: &CreateOfficeInput) -> Result<Self, sqlx::Error> {
        let slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.title),
        };

        let record = sqlx::query_as!(
            Office,
            r#"
                INSERT INTO office (slug, title, political_scope, state, municipality, district, district_type, chamber, election_scope, incumbent_id, office_type, term_length, seat)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING id, slug, title, office_type, district, district_type AS "district_type:District", chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", municipality, term_length, seat, created_at, updated_at
            "#,
            slug,
            input.title,
            input.political_scope as PoliticalScope,
            input.state as Option<State>,
            input.municipality,
            input.district,
            input.district_type as Option<District>,
            input.chamber as Option<Chamber>,
            input.election_scope as ElectionScope,
            input.incumbent_id,
            input.office_type,
            input.term_length,
            input.seat,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateOfficeInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Office,
            r#"
                UPDATE office
                SET slug = COALESCE($2, slug), 
                    title = COALESCE($3, title), 
                    political_scope = COALESCE($4, political_scope),
                    state = COALESCE($5, state),
                    municipality = COALESCE($6, municipality),
                    district = COALESCE($7, district),
                    district_type = COALESCE($8, district_type),
                    chamber = COALESCE($9, chamber),
                    election_scope = COALESCE($10, election_scope),
                    incumbent_id = COALESCE($11, incumbent_id),
                    office_type = COALESCE($12, office_type),
                    term_length = COALESCE($13, term_length),
                    seat = COALESCE($14, seat)
                WHERE id = $1
                RETURNING id, slug, title, office_type, district, district_type AS "district_type:District", chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", municipality, term_length, seat, created_at, updated_at
            "#,
            id,
            input.slug,
            input.title,
            input.political_scope as Option<PoliticalScope>,
            input.state as Option<State>,
            input.municipality,
            input.district,
            input.district_type as Option<District>,
            input.chamber as Option<Chamber>,
            input.election_scope as Option<ElectionScope>,
            input.incumbent_id,
            input.office_type,
            input.term_length,
            input.seat,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM office WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Office,
            r#"
                SELECT id, slug, title, office_type, district, district_type AS "district_type:District", chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", municipality, term_length, seat, created_at, updated_at FROM office
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
            Office,
            r#"
                SELECT id, slug, title, office_type, district, district_type AS "district_type:District", chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", municipality, term_length, seat, created_at, updated_at FROM office
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn search(db_pool: &PgPool, input: &OfficeSearch) -> Result<Vec<Self>, sqlx::Error> {
        let search_query =
            crate::process_search_query(input.query.to_owned().unwrap_or_else(|| "".to_string()));

        let records = sqlx::query_as!(
            Office,
            r#"
                SELECT id, slug, title, office_type, district, district_type AS "district_type:District", chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", municipality, term_length, seat, created_at, updated_at FROM office
                WHERE (($1::text = '') IS NOT FALSE OR to_tsvector(concat_ws(' ', slug, title)) @@ to_tsquery($1))
                AND ($2::state IS NULL OR state = $2)
                AND ($3::political_scope IS NULL OR political_scope = $3)
                
            "#,
            search_query,
            input.state as Option<State>,
            input.political_scope as Option<PoliticalScope>,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
