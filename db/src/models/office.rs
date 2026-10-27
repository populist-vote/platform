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
    pub subtitle: Option<String>,
    pub subtitle_short: Option<String>,
    pub name: Option<String>,
    pub office_type: Option<String>,
    pub district: Option<String>,
    /// Single-member subdivision of a school or hospital district.
    pub subdistrict: Option<String>,
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
    pub subdistrict: Option<String>,
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
    Transportation,
    Park,
    // Texas-specific (and potentially other states)
    BoardOfEducation,
    CourtOfAppeals,
    JusticeOfThePeace,
    Constable,
    VotingPrecinct,
    TxCollege,
    TxSpecial,
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

/// Maps a school or hospital parent plus an optional subdivision onto `district` and `subdistrict`.
/// A caller that already stored the subdivision in `subdistrict` is left unchanged.
fn stored_districts(input: &UpsertOfficeInput) -> (Option<String>, Option<String>) {
    let cleaned = |value: &Option<String>| {
        value
            .as_ref()
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
    };
    let district = cleaned(&input.district);
    let parent = cleaned(&input.school_district).or_else(|| cleaned(&input.hospital_district));
    let subdistrict = cleaned(&input.subdistrict);

    if subdistrict.is_some() {
        return (district, subdistrict);
    }
    match parent {
        Some(parent) if district.as_ref() == Some(&parent) => (Some(parent), None),
        Some(parent) if district.is_some() => (Some(parent), district),
        Some(parent) => (Some(parent), None),
        None => (district, None),
    }
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

        let (district, subdistrict) = stored_districts(input);

        let record = sqlx::query_as!(
            Office,
            r#"
                INSERT INTO office (id, slug, title, subtitle, subtitle_short, name, office_type, district, subdistrict, district_type, hospital_district, school_district, chamber, political_scope, election_scope, state, county, municipality, term_length, seat, priority)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
                ON CONFLICT (id) DO UPDATE
                SET
                    slug = COALESCE($2, office.slug),
                    title = COALESCE($3, office.title),
                    subtitle = COALESCE($4, office.subtitle),
                    subtitle_short = COALESCE($5, office.subtitle_short),
                    name = COALESCE($6, office.name),
                    office_type = COALESCE($7, office.office_type),
                    district = COALESCE($8, office.district),
                    subdistrict = COALESCE($9, office.subdistrict),
                    district_type = COALESCE($10, office.district_type),
                    hospital_district = COALESCE($11, office.hospital_district),
                    school_district = COALESCE($12, office.school_district),
                    chamber = COALESCE($13, office.chamber),
                    political_scope = COALESCE($14, office.political_scope),
                    election_scope = COALESCE($15, office.election_scope),
                    state = COALESCE($16, office.state),
                    county = COALESCE($17, office.county),
                    municipality = COALESCE($18, office.municipality),
                    term_length = COALESCE($19, office.term_length),
                    seat = COALESCE($20, office.seat),
                    priority = COALESCE($21, office.priority)
                RETURNING id, slug, title, subtitle, subtitle_short, name, office_type, district, subdistrict, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", political_scope AS "political_scope:PoliticalScope", election_scope as "election_scope:ElectionScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at
            "#,
            id,
            slug,
            input.title,
            input.subtitle,
            input.subtitle_short,
            input.name,
            input.office_type,
            district,
            subdistrict,
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

    pub async fn upsert_from_source(
        db_pool: &PgPool,
        input: &UpsertOfficeInput,
    ) -> Result<Self, sqlx::Error> {
        input
            .slug
            .as_ref()
            .ok_or("slug is required")
            .map_err(|err| sqlx::Error::AnyDriverError(err.into()))?;

        let (district, subdistrict) = stored_districts(input);

        sqlx::query_as!(
            Office,
            r#"
                INSERT INTO office (slug, title, subtitle, subtitle_short, name, office_type, district, subdistrict, district_type, hospital_district, school_district, chamber, political_scope, election_scope, state, county, municipality, term_length, seat, priority)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                ON CONFLICT (slug) DO UPDATE
                SET
                    title = COALESCE($2, office.title),
                    subtitle = COALESCE($3, office.subtitle),
                    subtitle_short = COALESCE($4, office.subtitle_short),
                    name = COALESCE($5, office.name),
                    office_type = COALESCE($6, office.office_type),
                    district = COALESCE($7, office.district),
                    subdistrict = COALESCE($8, office.subdistrict),
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
                RETURNING id, slug, title, subtitle, subtitle_short, name, office_type, district, subdistrict, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", political_scope AS "political_scope:PoliticalScope", election_scope as "election_scope:ElectionScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at
            "#,
            input.slug,
            input.title,
            input.subtitle,
            input.subtitle_short,
            input.name,
            input.office_type,
            district,
            subdistrict,
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
                SELECT id, slug, title, subtitle, subtitle_short, name, office_type, district, subdistrict, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at FROM office
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
                SELECT id, slug, title, subtitle, subtitle_short, name, office_type, district, subdistrict, district_type AS "district_type:DistrictType", hospital_district, school_district, chamber AS "chamber:Chamber", election_scope as "election_scope:ElectionScope", political_scope AS "political_scope:PoliticalScope", state AS "state:State", county, municipality, term_length, seat, priority, created_at, updated_at FROM office
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
                subdistrict,
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
                title || ' ' || name || ' ' || COALESCE(subtitle, '') || ' ' || COALESCE(office_type, '') || ' ' || COALESCE(district, '') || ' ' || COALESCE(subdistrict, '') || ' ' || COALESCE(hospital_district, '') || ' ' || COALESCE(school_district, '') || ' ' || COALESCE(state::text, '') || ' ' || COALESCE(county, '') || ' ' || COALESCE(municipality, '')
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

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        district: Option<&str>,
        subdistrict: Option<&str>,
        school: Option<&str>,
        hospital: Option<&str>,
    ) -> UpsertOfficeInput {
        UpsertOfficeInput {
            district: district.map(str::to_string),
            subdistrict: subdistrict.map(str::to_string),
            school_district: school.map(str::to_string),
            hospital_district: hospital.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn stores_a_subdivision_separately_from_its_parent() {
        let (district, subdistrict) =
            stored_districts(&input(Some("1"), None, None, Some("Cook County")));
        assert_eq!(district.as_deref(), Some("Cook County"));
        assert_eq!(subdistrict.as_deref(), Some("1"));
    }

    #[test]
    fn stores_a_parent_with_no_subdivision() {
        let (district, subdistrict) =
            stored_districts(&input(None, None, Some("Alamo Heights ISD"), None));
        assert_eq!(district.as_deref(), Some("Alamo Heights ISD"));
        assert_eq!(subdistrict, None);
    }

    #[test]
    fn prefers_the_school_parent_when_both_parents_are_set() {
        let (district, subdistrict) = stored_districts(&input(
            Some("2"),
            None,
            Some("Austin ISD"),
            Some("Cook County"),
        ));
        assert_eq!(district.as_deref(), Some("Austin ISD"));
        assert_eq!(subdistrict.as_deref(), Some("2"));
    }

    #[test]
    fn leaves_a_parent_that_is_already_stored_as_the_district() {
        let (district, subdistrict) = stored_districts(&input(
            Some("Alamo Heights ISD"),
            None,
            Some("Alamo Heights ISD"),
            None,
        ));
        assert_eq!(district.as_deref(), Some("Alamo Heights ISD"));
        assert_eq!(subdistrict, None);
    }

    #[test]
    fn keeps_an_explicit_subdistrict() {
        let (district, subdistrict) = stored_districts(&input(
            Some("Austin ISD"),
            Some("4"),
            Some("Austin ISD"),
            None,
        ));
        assert_eq!(district.as_deref(), Some("Austin ISD"));
        assert_eq!(subdistrict.as_deref(), Some("4"));
    }

    #[test]
    fn leaves_legislative_districts_unchanged() {
        let (district, subdistrict) = stored_districts(&input(Some("12"), None, None, None));
        assert_eq!(district.as_deref(), Some("12"));
        assert_eq!(subdistrict, None);
    }
}
