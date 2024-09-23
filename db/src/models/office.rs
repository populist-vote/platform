use super::enums::{PoliticalScope, State};
use crate::DateTime;
use async_graphql::{Enum, InputObject};
use rand::Rng;
use serde::{Deserialize, Serialize};
use slugify::slugify;
use sqlx::PgPool;
use strum_macros::{Display, EnumString};

#[derive(sqlx::FromRow, Debug, Clone, Default)]
pub struct Office {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub subtitle_short: Option<String>,
    pub name: Option<String>,
    pub office_type: Option<String>,
    pub district: Option<String>,
    pub district_type: Option<DistrictType>,
    pub hospital_district: Option<String>,
    pub school_district: Option<String>,
    pub chamber: Option<Chamber>,
    pub political_scope: PoliticalScope,
    /// Helps to determine which races to display to a user
    pub election_scope: ElectionScope,
    pub state: Option<State>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub term_length: Option<i32>,
    pub seat: Option<String>,
    pub priority: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Default, Serialize, Deserialize, InputObject)]
pub struct UpsertOfficeInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub subtitle_short: Option<String>,
    pub name: Option<String>,
    pub office_type: Option<String>,
    pub district: Option<String>,
    pub district_type: Option<DistrictType>,
    pub hospital_district: Option<String>,
    pub school_district: Option<String>,
    pub chamber: Option<Chamber>,
    pub election_scope: Option<ElectionScope>,
    pub political_scope: Option<PoliticalScope>,
    pub state: Option<State>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub term_length: Option<i32>,
    pub seat: Option<String>,
    pub priority: Option<i32>,
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
pub enum DistrictType {
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
    // California, Nevada, New Jersey, New York, Wisconsin only
    Assembly,
    // Nebraska only, it is unicameral
    Legislature,
}

#[derive(Default, Debug, Serialize, Deserialize, InputObject)]
pub struct OfficeFilter {
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
                INSERT INTO office (id, slug, title, subtitle, subtitle_short, name, office_type, district, district_type, hospital_district, school_district, chamber, political_scope, election_scope, state, county, municipality, term_length, seat, priority)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                ON CONFLICT (id) DO UPDATE
                SET
                    slug = COALESCE($2, office.slug),
                    title = COALESCE($3, office.title),
                    subtitle = COALESCE($4, office.subtitle),
                    subtitle_short = COALESCE($5, office.subtitle_short),
                    name = COALESCE($6, office.name),
                    office_type = COALESCE($7, office.office_type),
                    district = COALESCE($8, office.district),
                    district_type = COALESCE($9, office.district_type),
                    hospital_district = COALESCE($10, office.hospital_district),
                    school_district = COALESCE($11, office.school_district),
                    chamber = COALESCE($12, office.chamber),
                    political_scope = COALESCE($13, office.political_scope),
                    election_scope = COALESCE($14, office.election_scope),
                    state = COALESCE($15, office.state),
                    county = COALESCE($16, office.county),
                    municipality = COALESCE($17, office.municipality),
                    term_length = COALESCE($18, office.term_length),
                    seat = COALESCE($19, office.seat),
                    priority = COALESCE($20, office.priority)
                RETURNING id, slug, title, subtitle, subtitle_short, name, office_type, district, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", political_scope AS "political_scope:PoliticalScope", election_scope as "election_scope:ElectionScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at
            "#,
            id,
            slug,
            input.title,
            input.subtitle,
            input.subtitle_short,
            input.name,
            input.office_type,
            input.district,
            input.district_type as Option<DistrictType>,
            input.hospital_district,
            input.school_district,
            input.chamber as Option<Chamber>,
            input.political_scope as Option<PoliticalScope>,
            input.election_scope as Option<ElectionScope>,
            input.state as Option<State>,
            input.county,
            input.municipality,
            input.term_length,
            input.seat,
            input.priority,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    // TODO - Refactor this with the above upsert, maybe?
    pub async fn upsert_from_source(
        db_pool: &PgPool,
        input: &UpsertOfficeInput,
    ) -> Result<Self, sqlx::Error> {
        // FIXME - A better input interface would be ideal
        input
            .slug
            .as_ref()
            .ok_or("slug is required")
            .map_err(|err| sqlx::Error::AnyDriverError(err.into()))?;

        sqlx::query_as!(
            Office,
            r#"
                INSERT INTO office (slug, title, subtitle, subtitle_short, name, office_type, district, district_type, hospital_district, school_district, chamber, political_scope, election_scope, state, county, municipality, term_length, seat, priority)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                ON CONFLICT (slug) DO UPDATE
                SET
                    title = COALESCE($2, office.title),
                    subtitle = COALESCE($3, office.subtitle),
                    subtitle_short = COALESCE($4, office.subtitle_short),
                    name = COALESCE($5, office.name),
                    office_type = COALESCE($6, office.office_type),
                    district = COALESCE($7, office.district),
                    district_type = COALESCE($8, office.district_type),
                    hospital_district = COALESCE($9, office.hospital_district),
                    school_district = COALESCE($10, office.school_district),
                    chamber = COALESCE($11, office.chamber),
                    political_scope = COALESCE($12, office.political_scope),
                    election_scope = COALESCE($13, office.election_scope),
                    state = COALESCE($14, office.state),
                    county = COALESCE($15, office.county),
                    municipality = COALESCE($16, office.municipality),
                    term_length = COALESCE($17, office.term_length),
                    seat = COALESCE($18, office.seat),
                    priority = COALESCE($19, office.priority)
                RETURNING id, slug, title, subtitle, subtitle_short, name, office_type, district, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", political_scope AS "political_scope:PoliticalScope", election_scope as "election_scope:ElectionScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at
            "#,
            input.slug,
            input.title,
            input.subtitle,
            input.subtitle_short,
            input.name,
            input.office_type,
            input.district,
            input.district_type as Option<DistrictType>,
            input.hospital_district,
            input.school_district,
            input.chamber as Option<Chamber>,
            input.political_scope as Option<PoliticalScope>,
            input.election_scope as Option<ElectionScope>,
            input.state as Option<State>,
            input.county,
            input.municipality,
            input.term_length,
            input.seat,
            input.priority,
        )
        .fetch_one(db_pool)
        .await
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
                SELECT id, slug, title, subtitle, subtitle_short, name, office_type, district, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at FROM office
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
                SELECT id, slug, title, subtitle, subtitle_short, name, office_type, district, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at FROM office
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn filter(db_pool: &PgPool, input: &OfficeFilter) -> Result<Vec<Self>, sqlx::Error> {
        let search_query = input.query.to_owned().unwrap_or_default();

        let records = sqlx::query_as!(
            Office,
            r#"
            SELECT
                id,
                slug,
                title,
                subtitle,
                subtitle_short,
                name,
                office_type,
                district,
                district_type AS "district_type:DistrictType",
                hospital_district,
                school_district,
                chamber AS "chamber:Chamber",
                election_scope AS "election_scope:ElectionScope",
                political_scope AS "political_scope:PoliticalScope",
                state AS "state:State",
                county,
                municipality,
                term_length,
                seat,
                priority,
                created_at,
                updated_at
            FROM office,
            to_tsvector(
                title || ' ' || name || ' ' || COALESCE(subtitle, '') || ' ' || COALESCE(office_type, '') || ' ' || COALESCE(district, '') || ' ' || COALESCE(hospital_district, '') || ' ' || COALESCE(school_district, '') || ' ' || COALESCE(state::text, '') || ' ' || COALESCE(county, '') || ' ' || COALESCE(municipality, '')
            ) document,
            websearch_to_tsquery($1) query
            WHERE (($1::text = '') IS NOT FALSE OR query @@ document)
            AND($2::state IS NULL
                OR state = $2)
            AND($3::political_scope IS NULL
                OR political_scope = $3)
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
