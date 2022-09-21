use super::enums::{PoliticalScope, State};
use crate::DateTime;
use async_graphql::{Enum, InputObject};
use rand::Rng;
use serde::{Deserialize, Serialize};
use slugify::slugify;
use sqlx::PgPool;
use strum_macros::{Display, EnumString};

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Office {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub name: Option<String>,
    pub office_type: Option<String>,
    pub district: Option<String>,
    pub district_type: Option<District>,
    pub hospital_district: Option<String>,
    pub school_district: Option<String>,
    pub chamber: Option<Chamber>,
    pub political_scope: PoliticalScope,
    /// Helps to determine which races to display to a user
    pub election_scope: ElectionScope,
    pub state: Option<State>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    /// If a new office is introduced for redistricting or other reasons,
    /// there may not be an incumbent, hence optional
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
    Judicial,
    Hospital,
    SoilAndWater,
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
pub struct UpsertOfficeInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub name: Option<String>,
    pub office_type: Option<String>,
    pub district: Option<String>,
    pub district_type: Option<District>,
    pub hospital_district: Option<String>,
    pub school_district: Option<String>,
    pub chamber: Option<Chamber>,
    pub election_scope: Option<ElectionScope>,
    pub political_scope: Option<PoliticalScope>,
    pub state: Option<State>,
    pub county: Option<String>,
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
    pub async fn upsert(db_pool: &PgPool, input: &UpsertOfficeInput) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(uuid::Uuid::new_v4);

        let mut slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.title.clone().unwrap_or_default()),
        };

        let existing_slug = sqlx::query!(
            r#"
            SELECT slug
            FROM office
            WHERE slug = $1 AND id != $2
            "#,
            slug,
            input.id
        )
        .fetch_optional(db_pool)
        .await?;

        let rando: i32 = { rand::thread_rng().gen() };

        if let Some(r) = existing_slug {
            slug = format!("{}-{}", r.slug, rando);
        }

        let record = sqlx::query_as!(
            Office,
            r#"
                INSERT INTO office (id, slug, title, name, office_type, district, district_type, hospital_district, school_district, chamber, political_scope, election_scope, state, county, municipality, incumbent_id, term_length, seat)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                ON CONFLICT (id) DO UPDATE
                SET
                    slug = COALESCE($2, office.slug),
                    title = COALESCE($3, office.title),
                    name = COALESCE($4, office.name),
                    office_type = COALESCE($5, office.office_type),
                    district = COALESCE($6, office.district),
                    district_type = COALESCE($7, office.district_type),
                    hospital_district = COALESCE($8, office.hospital_district),
                    school_district = COALESCE($9, office.school_district),
                    chamber = COALESCE($10, office.chamber),
                    political_scope = COALESCE($11, office.political_scope),
                    election_scope = COALESCE($12, office.election_scope),
                    state = COALESCE($13, office.state),
                    county = COALESCE($14, office.county),
                    municipality = COALESCE($15, office.municipality),
                    incumbent_id = COALESCE($16, office.incumbent_id),
                    term_length = COALESCE($17, office.term_length),
                    seat = COALESCE($18, office.seat)
                RETURNING id, slug, title, name, office_type, district, district_type AS "district_type:District", hospital_district, school_district, chamber AS "chamber:Chamber", political_scope AS "political_scope:PoliticalScope", election_scope as "election_scope:ElectionScope", state AS "state:State", county, municipality, incumbent_id, term_length, seat, created_at, updated_at
            "#,
            id,
            slug,
            input.title,
            input.name,
            input.office_type,
            input.district,
            input.district_type as Option<District>,
            input.hospital_district,
            input.school_district,
            input.chamber as Option<Chamber>,
            input.political_scope as Option<PoliticalScope>,
            input.election_scope as Option<ElectionScope>,
            input.state as Option<State>,
            input.county,
            input.municipality,
            input.incumbent_id,
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
                SELECT id, slug, title, name, office_type, district, district_type AS "district_type:District", hospital_district, school_district, chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", county, municipality, term_length, seat, created_at, updated_at FROM office
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
                SELECT id, slug, title, name, office_type, district, district_type AS "district_type:District", hospital_district, school_district, chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", county, municipality, term_length, seat, created_at, updated_at FROM office
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
                SELECT id, slug, title, name, office_type, district, district_type AS "district_type:District",  hospital_district, school_district, chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", incumbent_id, state AS "state:State", county, municipality, term_length, seat, created_at, updated_at FROM office
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
