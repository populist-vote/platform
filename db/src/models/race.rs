use super::enums::{PoliticalParty, PoliticalScope, RaceType, State, VoteType};
use crate::{DateTime, ElectionScope};
use async_graphql::InputObject;
use chrono::NaiveDate;
use itertools::Itertools;
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
    pub vote_type: VoteType,
    pub party: Option<PoliticalParty>,
    pub state: Option<State>,
    pub description: Option<String>,
    pub ballotpedia_link: Option<String>,
    pub early_voting_begins_date: Option<NaiveDate>,
    pub official_website: Option<String>,
    pub election_id: Option<uuid::Uuid>,
    pub winner_ids: Option<Vec<uuid::Uuid>>,
    pub total_votes: Option<i32>,
    pub is_special_election: bool,
    pub num_elect: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Default, Serialize, Deserialize, InputObject)]
pub struct UpsertRaceInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub office_id: Option<uuid::Uuid>,
    pub race_type: Option<RaceType>,
    pub vote_type: Option<VoteType>,
    pub party: Option<PoliticalParty>,
    pub description: Option<String>,
    pub ballotpedia_link: Option<String>,
    pub early_voting_begins_date: Option<NaiveDate>,
    pub official_website: Option<String>,
    pub state: Option<State>,
    pub election_id: Option<uuid::Uuid>,
    pub winner_ids: Option<Vec<uuid::Uuid>>,
    pub total_votes: Option<i32>,
    pub is_special_election: bool,
    pub num_elect: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize, InputObject)]
pub struct RaceFilter {
    query: Option<String>,
    state: Option<State>,
    political_scope: Option<PoliticalScope>,
    election_scope: Option<ElectionScope>,
    office_titles: Option<Vec<String>>,
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
                INSERT INTO race (id, slug, title, office_id, race_type, vote_type, party, state,  description, ballotpedia_link, early_voting_begins_date, winner_ids, official_website, election_id, total_votes, is_special_election, num_elect)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                ON CONFLICT (id) DO UPDATE
                SET
                    slug = COALESCE($2, race.slug),
                    title = COALESCE($3, race.title), 
                    office_id = COALESCE($4, race.office_id),
                    race_type = COALESCE($5, race.race_type),
                    vote_type = COALESCE($6, race.vote_type),
                    party = COALESCE($7, race.party),
                    state = COALESCE($8, race.state),
                    description = COALESCE($9, race.description),
                    ballotpedia_link = COALESCE($10, race.ballotpedia_link),
                    early_voting_begins_date = COALESCE($11, race.early_voting_begins_date),
                    winner_ids = COALESCE($12, race.winner_ids),
                    official_website = COALESCE($13, race.official_website),
                    election_id = COALESCE($14, race.election_id),
                    total_votes = COALESCE($15, race.total_votes),
                    is_special_election = COALESCE($16, race.is_special_election),
                    num_elect = COALESCE($17, race.num_elect)
                RETURNING id, slug, title,  office_id, race_type AS "race_type:RaceType", vote_type AS "vote_type:VoteType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, winner_ids, official_website, election_id, total_votes, is_special_election, num_elect, created_at, updated_at
            "#,
            id,
            slug,
            input.title,
            input.office_id,
            input.race_type as Option<RaceType>,
            input.vote_type as Option<VoteType>,
            input.party as Option<PoliticalParty>,
            input.state as Option<State>,
            input.description,
            input.ballotpedia_link,
            input.early_voting_begins_date,
            input.winner_ids.as_ref().map(|v| v.as_slice()),
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
                SELECT id, slug, title, office_id, race_type AS "race_type:RaceType", vote_type AS "vote_type:VoteType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, winner_ids, total_votes, official_website, election_id,  is_special_election, num_elect, created_at, updated_at FROM race
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
                SELECT id, slug, title, office_id, race_type AS "race_type:RaceType", vote_type AS "vote_type:VoteType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, winner_ids, total_votes, official_website, election_id, is_special_election, num_elect, created_at, updated_at FROM race
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn filter(db_pool: &PgPool, input: RaceFilter) -> Result<Vec<Self>, sqlx::Error> {
        let office_titles = match input.office_titles {
            Some(office_titles) => office_titles.iter().map(|t| format!("'{}'", t)).join(","),
            None => "NULL".to_string(),
        };

        let query = &*format!(
            r#"
                SELECT
                    race.id,
                    race.slug,
                    race.title,
                    office_id,
                    race_type,
                    vote_type,
                    party,
                    race.state,
                    race.description,
                    ballotpedia_link,
                    early_voting_begins_date,
                    winner_ids,
                    total_votes,
                    official_website,
                    election_id,
                    is_special_election,
                    num_elect,
                    race.created_at,
                    race.updated_at,
                    o.title,
                    o.state,
                    o.county,
                    o.municipality,
                    o.district,
                    o.school_district,
                    o.hospital_district
                FROM
                    race
                LEFT JOIN office o ON office_id = o.id
                JOIN election e ON election_id = e.id
                JOIN us_states s ON race.state = s.code,
                to_tsvector(
                    COALESCE(race.title, '') || ' ' ||
                    COALESCE(e.election_date::text, '') || ' ' ||
                    COALESCE(o.title, '') || ' ' ||
                    COALESCE(o.state::text, '') || ' ' ||
                    COALESCE(s.name::text, '') || ' ' ||
                    COALESCE(o.county, '') || ' ' ||
                    COALESCE(o.municipality, '') || ' ' ||
                    COALESCE(o.district, '') || ' ' ||
                    COALESCE(o.school_district, '') || ' ' ||
                    COALESCE(o.hospital_district, '')
                ) document,
                websearch_to_tsquery({query}::text) query,
                NULLIF(ts_rank(to_tsvector(race.title), query), 0) AS rank_race_title,
                NULLIF(ts_rank(to_tsvector(e.election_date::text), query), 0) AS rank_election_date,
                NULLIF(ts_rank(to_tsvector(o.title), query), 0) AS rank_office_title,
                NULLIF(ts_rank(to_tsvector(o.state::text), query), 0) AS rank_office_state_code,
                NULLIF(ts_rank(to_tsvector(s.name::text), query), 0) AS rank_office_state_full,
                NULLIF(ts_rank(to_tsvector(o.county), query), 0) AS rank_office_county,
                NULLIF(ts_rank(to_tsvector(o.municipality), query), 0) AS rank_office_municipality,
                NULLIF(ts_rank(to_tsvector(o.district), query), 0) AS rank_office_district,
                NULLIF(ts_rank(to_tsvector(o.school_district), query), 0) AS rank_office_school_district,
                NULLIF(ts_rank(to_tsvector(o.hospital_district), query), 0) AS rank_office_hospital_district
                WHERE (({query}::text = '') IS NOT FALSE OR query @@ document)
                AND ({state} IS NULL
                    OR race.state = {state})
                AND({political_scope} IS NULL
                    OR o.political_scope = {political_scope})
                AND({election_scope} IS NULL
                    OR o.election_scope = {election_scope})
                AND(({office_titles}) IS NULL
                    OR o.title IN ({office_titles}))
                ORDER BY e.election_date DESC, o.priority ASC, o.district ASC, o.title DESC
                LIMIT 50
                "#,
            query = input
                .query
                .map(|s| format!("'{}'", s))
                .unwrap_or_else(|| "NULL".to_string()),
            state = input
                .state
                .map(|s| format!("'{}'", s))
                .unwrap_or_else(|| "NULL".to_string()),
            political_scope = input
                .political_scope
                .map(|s| format!("'{}'", s))
                .unwrap_or_else(|| "NULL".to_string()),
            election_scope = input
                .election_scope
                .map(|s| format!("'{}'", s))
                .unwrap_or_else(|| "NULL".to_string()),
            office_titles = office_titles
        );

        let records = sqlx::query_as(query).fetch_all(db_pool).await?;

        Ok(records)
    }
}
