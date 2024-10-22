use super::enums::{BallotMeasureStatus, State};
use crate::{DateTime, ElectionScope, IssueTag, PoliticalScope, PopularitySort};
use async_graphql::InputObject;
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Debug, Clone)]
pub struct BallotMeasure {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub status: BallotMeasureStatus,
    pub election_id: uuid::Uuid,
    pub state: State,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub school_district: Option<String>,
    pub ballot_measure_code: String,
    pub county_fips: Option<String>,
    pub municipality_fips: Option<String>,
    pub measure_type: Option<String>, //perhaps make enum later
    pub definitions: Option<String>,  // markdown list of bulleted items
    pub yes_votes: Option<i32>,
    pub no_votes: Option<i32>,
    pub num_precincts_reporting: Option<i32>,
    pub total_precincts: Option<i32>,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub election_scope: Option<ElectionScope>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct UpsertBallotMeasureInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub election_id: Option<uuid::Uuid>,
    pub title: Option<String>,
    pub status: Option<BallotMeasureStatus>,
    pub state: Option<State>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub school_district: Option<String>,
    pub ballot_measure_code: Option<String>,
    pub measure_type: Option<String>,
    pub definitions: Option<String>,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub county_fips: Option<String>,
    pub municipality_fips: Option<String>,
    pub election_scope: Option<ElectionScope>,
}

#[derive(InputObject, Default, Debug)]
pub struct BallotMeasureFilter {
    query: Option<String>,
    political_scope: Option<PoliticalScope>,
    state: Option<State>,
    year: Option<i32>,
    status: Option<BallotMeasureStatus>,
    issue_tag: Option<String>,
}

#[derive(InputObject, Default, Debug)]
pub struct BallotMeasureSort {
    popularity: Option<PopularitySort>,
}

impl BallotMeasure {
    pub async fn upsert(
        db_pool: &PgPool,
        input: &UpsertBallotMeasureInput,
    ) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let record = sqlx::query_as!(
            BallotMeasure,
            r#"
                INSERT INTO ballot_measure 
                (id, election_id, slug, title, status, description, official_summary, 
                populist_summary, full_text_url, state, ballot_measure_code, 
                measure_type, definitions, county_fips, municipality_fips, county, school_district, election_scope) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                ON CONFLICT (id) DO UPDATE SET
                    slug = COALESCE($3, ballot_measure.slug),
                    title = COALESCE($4, ballot_measure.title),
                    status = COALESCE($5, ballot_measure.status),
                    description = COALESCE($6, ballot_measure.description),
                    official_summary = COALESCE($7, ballot_measure.official_summary),
                    populist_summary = COALESCE($8, ballot_measure.populist_summary),
                    full_text_url = COALESCE($9, ballot_measure.full_text_url),
                    state = COALESCE($10, ballot_measure.state),
                    ballot_measure_code = COALESCE($11, ballot_measure.ballot_measure_code),
                    measure_type = COALESCE($12, ballot_measure.measure_type),
                    definitions = COALESCE($13, ballot_measure.definitions),
                    county_fips = COALESCE($14, ballot_measure.county_fips),
                    municipality_fips = COALESCE($15, ballot_measure.municipality_fips),
                    county = COALESCE($16, ballot_measure.county),
                    school_district = COALESCE($17, ballot_measure.school_district),
                    election_scope = COALESCE($18, ballot_measure.election_scope)
                RETURNING id, election_id, slug, title, status AS "status: BallotMeasureStatus", description, official_summary, populist_summary, full_text_url, state AS "state:State", county, municipality, school_district, ballot_measure_code, measure_type, definitions, yes_votes, no_votes, num_precincts_reporting, total_precincts, county_fips, municipality_fips, election_scope AS "election_scope:ElectionScope", created_at, updated_at
            "#,
            id,
            input.election_id,
            input.slug,
            input.title,
            input.status as Option<BallotMeasureStatus>,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.state as Option<State>,
            input.ballot_measure_code,
            input.measure_type,
            input.definitions,
            input.county_fips,
            input.municipality_fips,
            input.county,
            input.school_district,
            input.election_scope as Option<ElectionScope>
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn upsert_from_source(
        db_pool: &PgPool,
        input: &UpsertBallotMeasureInput,
    ) -> Result<Self, sqlx::Error> {
        input
            .slug
            .as_ref()
            .ok_or("slug is required")
            .map_err(|err| sqlx::Error::AnyDriverError(err.into()))?;

        sqlx::query_as!(
            BallotMeasure,
            r#"
                INSERT INTO ballot_measure 
                (slug, election_id, title, status, description, official_summary, 
                populist_summary, full_text_url, state, ballot_measure_code, 
                measure_type, definitions, county_fips, municipality_fips, county, municipality, school_district, election_scope) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                ON CONFLICT (slug) DO UPDATE SET
                    title = COALESCE($3, ballot_measure.title),
                    status = COALESCE($4, ballot_measure.status),
                    description = COALESCE($5, ballot_measure.description),
                    official_summary = COALESCE($6, ballot_measure.official_summary),
                    populist_summary = COALESCE($7, ballot_measure.populist_summary),
                    full_text_url = COALESCE($8, ballot_measure.full_text_url),
                    state = COALESCE($9, ballot_measure.state),
                    ballot_measure_code = COALESCE($10, ballot_measure.ballot_measure_code),
                    measure_type = COALESCE($11, ballot_measure.measure_type),
                    definitions = COALESCE($12, ballot_measure.definitions),
                    county_fips = COALESCE($13, ballot_measure.county_fips),
                    municipality_fips = COALESCE($14, ballot_measure.municipality_fips),
                    county = COALESCE($15, ballot_measure.county),
                    municipality = COALESCE($16, ballot_measure.municipality),
                    school_district = COALESCE($17, ballot_measure.school_district),
                    election_scope = COALESCE($18, ballot_measure.election_scope)
                RETURNING id, election_id, slug, title, status AS "status: BallotMeasureStatus", description, official_summary, populist_summary, full_text_url, state AS "state:State", county, municipality, school_district, ballot_measure_code, measure_type, definitions, yes_votes, no_votes, num_precincts_reporting, total_precincts, county_fips, municipality_fips, election_scope AS "election_scope:ElectionScope", created_at, updated_at
            "#,
            input.slug,
            input.election_id,
            input.title,
            input.status as Option<BallotMeasureStatus>,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.state as Option<State>,
            input.ballot_measure_code,
            input.measure_type,
            input.definitions,
            input.county_fips,
            input.municipality_fips,
            input.county,
            input.municipality,
            input.school_district,
            input.election_scope as Option<ElectionScope>
        )
        .fetch_one(db_pool)
        .await
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ballot_measure WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            BallotMeasure,
            r#"
                SELECT 
                    ballot_measure.id, 
                    election_id, 
                    ballot_measure.slug, 
                    ballot_measure.title, 
                    ballot_measure.status AS "status: BallotMeasureStatus", 
                    ballot_measure.state AS "state:State", 
                    ballot_measure_code, 
                    measure_type, 
                    definitions, 
                    ballot_measure.description, 
                    official_summary, 
                    populist_summary, 
                    full_text_url, 
                    yes_votes, 
                    no_votes, 
                    num_precincts_reporting, 
                    total_precincts, 
                    county_fips,
                    municipality_fips,
                    county, 
                    municipality,
                    school_district,
                    election_scope AS "election_scope:ElectionScope",
                    ballot_measure.created_at, 
                    ballot_measure.updated_at 
                FROM ballot_measure 
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(record)
    }

    pub async fn filter(
        db_pool: &PgPool,
        filter: &BallotMeasureFilter,
        sort: &BallotMeasureSort,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let query = format!(
            r#"
                SELECT 
                    ballot_measure.id, 
                    election_id, 
                    ballot_measure.slug, 
                    ballot_measure.title, 
                    ballot_measure.status, 
                    ballot_measure.state, 
                    ballot_measure_code, 
                    measure_type, 
                    definitions, 
                    ballot_measure.description, 
                    official_summary, 
                    populist_summary, 
                    full_text_url, 
                    yes_votes, 
                    no_votes, 
                    num_precincts_reporting, 
                    total_precincts, 
                    ballot_measure.created_at, 
                    ballot_measure.updated_at 
                FROM ballot_measure
                LEFT JOIN ballot_measure_public_votes bpv ON bpv.ballot_measure_id = ballot_measure.id
                JOIN election e ON e.id = ballot_measure.election_id,
                LATERAL (
                    SELECT
                    ARRAY (
                        SELECT
                        t.slug
                        FROM
                        ballot_measure_issue_tags bit
                        JOIN issue_tag t ON t.id = bit.issue_tag_id
                        WHERE
                        bit.ballot_measure_id = ballot_measure.id
                    ) AS tag_array
                ) t,
                to_tsvector(ballot_measure.title || ' ' || ballot_measure_code || ' ' || COALESCE(ballot_measure.description, '')) document,
                websearch_to_tsquery ($1::text) query,
                NULLIF(ts_rank(to_tsvector(ballot_measure_code), query), 0) rank_ballot_measure_code,
                NULLIF(ts_rank(to_tsvector(ballot_measure.title), query), 0) rank_title,
                NULLIF(ts_rank(to_tsvector(ballot_measure.description), query), 0) rank_description
                WHERE ($1::text IS NULL OR document @@ query)
                AND($2::ballot_measure_status IS NULL OR ballot_measure.status = $2)
                -- TODO: Add political scope to the query
                AND(
                    ($3::state IS NULL OR ballot_measure.state = $3)
                )
                AND ($4::integer IS NULL OR EXTRACT(YEAR FROM e.election_date) = $4)
                AND ($5::text IS NULL OR $5::text = ANY(t.tag_array))
                GROUP BY
                (
                    ballot_measure.id,
                    rank_ballot_measure_code,
                    rank_title,
                    rank_description,
                    t.tag_array
                )
                ORDER BY {order_by}
                LIMIT 20
                
            "#,
            order_by = match sort.popularity {
                Some(PopularitySort::MostPopular) => "rank_ballot_measure_code, rank_title, rank_description, COUNT(bpv.*) DESC NULLS LAST",
                Some(PopularitySort::MostSupported) =>
                    "rank_ballot_measure_code, rank_title, rank_description, SUM(CASE WHEN bpv.position = 'support' THEN 1 ELSE 0 END)DESC NULLS LAST",
                Some(PopularitySort::MostOpposed) =>
                    "rank_ballot_measure_code, rank_title, rank_description, SUM(CASE WHEN bpv.position = 'oppose' THEN 1 ELSE 0 END) DESC NULLS LAST",
                None => "rank_ballot_measure_code, rank_title, rank_description DESC NULLS LAST",
            }
        );

        let records = sqlx::query_as::<_, BallotMeasure>(&query)
            .bind(filter.query.to_owned())
            .bind(filter.status)
            .bind(filter.state)
            .bind(filter.year)
            .bind(filter.issue_tag.to_owned())
            .fetch_all(db_pool)
            .await?;
        Ok(records)
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN ballot_measure_issue_tags bmit
                ON bmit.issue_tag_id = it.id
                WHERE bmit.ballot_measure_id = $1
            "#,
            bill_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
