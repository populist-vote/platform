use async_trait::async_trait;
use axum::{
    extract::{rejection::QueryRejection, Path, Query, State},
    http::{header::CACHE_CONTROL, HeaderMap, HeaderValue},
    Json,
};
use db::{
    BallotMeasure, DistrictType, Election, ElectionScope, PoliticalScope, Race, RaceType,
    State as UsState, VoteType,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

use super::{
    ballots::{assemble_full_race_resource, RaceResource},
    error::ApiError,
    pagination::{DEFAULT_LIMIT, MAX_LIMIT},
    RestState,
};

const ELECTIONS_ENDPOINT: &str = "/api/v1/elections";
const CACHE_METADATA: HeaderValue =
    HeaderValue::from_static("public, max-age=300, stale-while-revalidate=900");
const CACHE_COLLECTION: HeaderValue =
    HeaderValue::from_static("public, max-age=60, stale-while-revalidate=300");
const CACHE_RESULTS: HeaderValue =
    HeaderValue::from_static("public, max-age=5, stale-while-revalidate=25");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PageRequest {
    pub(super) limit: usize,
    pub(super) offset: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct ElectionFilters {
    pub(super) state: Option<UsState>,
    pub(super) year: Option<i32>,
    pub(super) query: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct RaceFilters {
    pub(super) state: Option<UsState>,
    pub(super) race_type: Option<RaceType>,
    pub(super) political_scope: Option<PoliticalScope>,
    pub(super) election_scope: Option<ElectionScope>,
    pub(super) district_type: Option<DistrictType>,
    pub(super) query: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct BallotMeasureFilters {
    pub(super) state: Option<UsState>,
    pub(super) status: Option<&'static str>,
    pub(super) election_scope: Option<ElectionScope>,
    pub(super) county: Option<String>,
    pub(super) municipality: Option<String>,
    pub(super) school_district: Option<String>,
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub(super) enum ElectionDataError {
    #[error("election not found")]
    ElectionNotFound,
    #[error("race not found")]
    RaceNotFound,
    #[error("election data lookup failed")]
    Internal,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ElectionResource {
    pub(super) id: String,
    pub(super) slug: String,
    pub(super) title: String,
    pub(super) description: Option<String>,
    pub(super) state: Option<UsState>,
    pub(super) municipality: Option<String>,
    pub(super) election_date: chrono::NaiveDate,
}

impl From<Election> for ElectionResource {
    fn from(election: Election) -> Self {
        Self {
            id: election.id.to_string(),
            slug: election.slug,
            title: election.title,
            description: election.description,
            state: election.state,
            municipality: election.municipality,
            election_date: election.election_date,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OfficeSummaryResource {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) subtitle: Option<String>,
    pub(super) state: Option<UsState>,
    pub(super) county: Option<String>,
    pub(super) municipality: Option<String>,
    pub(super) district: Option<String>,
    pub(super) seat: Option<String>,
    pub(super) election_scope: ElectionScope,
    pub(super) district_type: Option<DistrictType>,
    pub(super) political_scope: PoliticalScope,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PartySummaryResource {
    pub(super) id: String,
    pub(super) slug: String,
    pub(super) name: String,
    pub(super) fec_code: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ResultSummaryResource {
    pub(super) total_votes: Option<i32>,
    pub(super) num_precincts_reporting: Option<i32>,
    pub(super) total_precincts: Option<i32>,
    pub(super) precinct_reporting_percentage: Option<f64>,
    pub(super) winners: Vec<IdResource>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct IdResource {
    pub(super) id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RaceSummaryResource {
    pub(super) id: String,
    pub(super) slug: String,
    pub(super) title: String,
    pub(super) election_id: String,
    pub(super) race_type: RaceType,
    pub(super) vote_type: VoteType,
    pub(super) state: Option<UsState>,
    pub(super) is_special_election: bool,
    pub(super) num_elect: Option<i32>,
    pub(super) office: OfficeSummaryResource,
    pub(super) party: Option<PartySummaryResource>,
    pub(super) results: ResultSummaryResource,
    pub(super) updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RaceDetailResource {
    #[serde(flatten)]
    pub(super) race: RaceResource,
    pub(super) description: Option<String>,
    pub(super) ballotpedia_link: Option<String>,
    pub(super) early_voting_begins_date: Option<chrono::NaiveDate>,
    pub(super) official_website: Option<String>,
    pub(super) updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CandidateResultResource {
    pub(super) id: String,
    pub(super) slug: String,
    pub(super) full_name: String,
    pub(super) party: Option<PartySummaryResource>,
    pub(super) votes: Option<i32>,
    pub(super) vote_percentage: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RaceResultsFeedResource {
    pub(super) race_id: String,
    pub(super) slug: String,
    pub(super) title: String,
    pub(super) office: OfficeSummaryResource,
    pub(super) total_votes: Option<i32>,
    pub(super) num_precincts_reporting: Option<i32>,
    pub(super) total_precincts: Option<i32>,
    pub(super) precinct_reporting_percentage: Option<f64>,
    pub(super) candidates: Vec<CandidateResultResource>,
    pub(super) winners: Vec<IdResource>,
    pub(super) updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BallotMeasureResource {
    pub(super) id: String,
    pub(super) slug: String,
    pub(super) title: String,
    pub(super) status: &'static str,
    pub(super) election_id: String,
    pub(super) state: UsState,
    pub(super) county: Option<String>,
    pub(super) municipality: Option<String>,
    pub(super) school_district: Option<String>,
    pub(super) ballot_measure_code: String,
    pub(super) measure_type: Option<String>,
    pub(super) definitions: Option<String>,
    pub(super) description: Option<String>,
    pub(super) official_summary: Option<String>,
    pub(super) populist_summary: Option<String>,
    pub(super) full_text_url: Option<String>,
    pub(super) yes_votes: Option<i32>,
    pub(super) no_votes: Option<i32>,
    pub(super) num_precincts_reporting: Option<i32>,
    pub(super) total_precincts: Option<i32>,
    pub(super) election_scope: Option<ElectionScope>,
    pub(super) updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<BallotMeasure> for BallotMeasureResource {
    fn from(measure: BallotMeasure) -> Self {
        Self {
            id: measure.id.to_string(),
            slug: measure.slug,
            title: measure.title,
            status: ballot_measure_status_name(measure.status),
            election_id: measure.election_id.to_string(),
            state: measure.state,
            county: measure.county,
            municipality: measure.municipality,
            school_district: measure.school_district,
            ballot_measure_code: measure.ballot_measure_code,
            measure_type: measure.measure_type,
            definitions: measure.definitions,
            description: measure.description,
            official_summary: measure.official_summary,
            populist_summary: measure.populist_summary,
            full_text_url: measure.full_text_url,
            yes_votes: measure.yes_votes,
            no_votes: measure.no_votes,
            num_precincts_reporting: measure.num_precincts_reporting,
            total_precincts: measure.total_precincts,
            election_scope: measure.election_scope,
            updated_at: measure.updated_at,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct PaginationMeta {
    pub(super) count: usize,
    pub(super) total: usize,
    pub(super) limit: usize,
    pub(super) offset: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct Collection<T> {
    pub(super) data: Vec<T>,
    pub(super) meta: PaginationMeta,
}

impl<T> Collection<T> {
    pub(super) fn new(data: Vec<T>, total: usize, page: PageRequest) -> Self {
        Self {
            meta: PaginationMeta {
                count: data.len(),
                total,
                limit: page.limit,
                offset: page.offset,
            },
            data,
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Resource<T> {
    pub(super) data: T,
}

#[async_trait]
pub(super) trait ElectionDataLookup: Send + Sync {
    async fn elections(
        &self,
        filters: ElectionFilters,
        page: PageRequest,
    ) -> Result<Collection<ElectionResource>, ElectionDataError>;

    async fn election(&self, election_id: Uuid) -> Result<ElectionResource, ElectionDataError>;

    async fn races(
        &self,
        election_id: Uuid,
        filters: RaceFilters,
        page: PageRequest,
    ) -> Result<Collection<RaceSummaryResource>, ElectionDataError>;

    async fn race(
        &self,
        election_id: Uuid,
        race_id: Uuid,
        endorser_id: Option<Uuid>,
    ) -> Result<RaceDetailResource, ElectionDataError>;

    async fn results(
        &self,
        election_id: Uuid,
        page: PageRequest,
    ) -> Result<Collection<RaceResultsFeedResource>, ElectionDataError>;

    async fn ballot_measures(
        &self,
        election_id: Uuid,
        filters: BallotMeasureFilters,
        page: PageRequest,
    ) -> Result<Collection<BallotMeasureResource>, ElectionDataError>;
}

pub(super) struct DatabaseElectionDataLookup {
    pool: PgPool,
}

impl DatabaseElectionDataLookup {
    pub(super) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn require_election(&self, election_id: Uuid) -> Result<Election, ElectionDataError> {
        Election::find_by_id(&self.pool, election_id)
            .await
            .map_err(map_election_error)
    }
}

#[async_trait]
impl ElectionDataLookup for DatabaseElectionDataLookup {
    async fn elections(
        &self,
        filters: ElectionFilters,
        page: PageRequest,
    ) -> Result<Collection<ElectionResource>, ElectionDataError> {
        let mut count =
            QueryBuilder::<Postgres>::new("SELECT COUNT(*)::bigint FROM election e WHERE TRUE");
        apply_election_filters(&mut count, &filters);
        let total = count
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(log_internal)?
            .max(0) as usize;

        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                e.id, e.slug, e.title, e.description, e.state, e.municipality,
                e.election_date
            FROM election e
            WHERE TRUE
            "#,
        );
        apply_election_filters(&mut query, &filters);
        query.push(" ORDER BY e.election_date DESC, e.title ASC, e.id ASC LIMIT ");
        query.push_bind(page.limit as i64);
        query.push(" OFFSET ");
        query.push_bind(page.offset as i64);
        let data = query
            .build_query_as::<Election>()
            .fetch_all(&self.pool)
            .await
            .map_err(log_internal)?
            .into_iter()
            .map(ElectionResource::from)
            .collect();
        Ok(Collection::new(data, total, page))
    }

    async fn election(&self, election_id: Uuid) -> Result<ElectionResource, ElectionDataError> {
        self.require_election(election_id)
            .await
            .map(ElectionResource::from)
    }

    async fn races(
        &self,
        election_id: Uuid,
        filters: RaceFilters,
        page: PageRequest,
    ) -> Result<Collection<RaceSummaryResource>, ElectionDataError> {
        self.require_election(election_id).await?;
        let mut count = QueryBuilder::<Postgres>::new(
            "SELECT COUNT(*)::bigint FROM race r JOIN office o ON o.id = r.office_id WHERE r.election_id = ",
        );
        count.push_bind(election_id);
        apply_race_filters(&mut count, &filters);
        let total = count
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(log_internal)?
            .max(0) as usize;

        let mut query = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                r.id, r.slug, r.title, r.election_id, r.race_type, r.vote_type,
                r.state, r.is_special_election, r.num_elect, r.total_votes,
                r.num_precincts_reporting, r.total_precincts, r.winner_ids,
                r.updated_at,
                o.id AS office_id, o.title AS office_title, o.subtitle AS office_subtitle,
                o.state AS office_state, o.county AS office_county,
                o.municipality AS office_municipality, o.district AS office_district,
                o.seat AS office_seat, o.election_scope, o.district_type,
                o.political_scope,
                p.id AS party_id, p.slug AS party_slug, p.name AS party_name,
                p.fec_code AS party_fec_code
            FROM race r
            JOIN office o ON o.id = r.office_id
            LEFT JOIN party p ON p.id = r.party_id
            WHERE r.election_id =
            "#,
        );
        query.push_bind(election_id);
        apply_race_filters(&mut query, &filters);
        push_race_order(&mut query);
        query.push(" LIMIT ");
        query.push_bind(page.limit as i64);
        query.push(" OFFSET ");
        query.push_bind(page.offset as i64);
        let data = query
            .build_query_as::<RaceDirectoryRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(log_internal)?
            .into_iter()
            .map(RaceSummaryResource::from)
            .collect();
        Ok(Collection::new(data, total, page))
    }

    async fn race(
        &self,
        election_id: Uuid,
        race_id: Uuid,
        endorser_id: Option<Uuid>,
    ) -> Result<RaceDetailResource, ElectionDataError> {
        let election = self.require_election(election_id).await?;
        let race = Race::find_by_id(&self.pool, race_id)
            .await
            .map_err(map_race_error)?;
        if race.election_id != Some(election_id) {
            return Err(ElectionDataError::RaceNotFound);
        }
        let description = race.description.clone();
        let ballotpedia_link = race.ballotpedia_link.clone();
        let early_voting_begins_date = race.early_voting_begins_date;
        let official_website = race.official_website.clone();
        let updated_at = race.updated_at;
        let race =
            assemble_full_race_resource(&self.pool, race, election.election_date, endorser_id)
                .await
                .map_err(log_internal)?;
        Ok(RaceDetailResource {
            race,
            description,
            ballotpedia_link,
            early_voting_begins_date,
            official_website,
            updated_at,
        })
    }

    async fn results(
        &self,
        election_id: Uuid,
        page: PageRequest,
    ) -> Result<Collection<RaceResultsFeedResource>, ElectionDataError> {
        self.require_election(election_id).await?;
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::bigint FROM race WHERE election_id = $1",
        )
        .bind(election_id)
        .fetch_one(&self.pool)
        .await
        .map_err(log_internal)?
        .max(0) as usize;
        let rows = sqlx::query_as::<_, ResultsRow>(
            r#"
            WITH selected_races AS (
                SELECT
                    r.id, r.slug, r.title, r.total_votes, r.num_precincts_reporting,
                    r.total_precincts, r.winner_ids, r.updated_at,
                    o.id AS office_id, o.title AS office_title,
                    o.subtitle AS office_subtitle, o.state AS office_state,
                    o.county AS office_county, o.municipality AS office_municipality,
                    o.district AS office_district, o.seat AS office_seat,
                    o.election_scope, o.district_type, o.political_scope,
                    o.priority
                FROM race r
                JOIN office o ON o.id = r.office_id
                WHERE r.election_id = $1
                ORDER BY
                    o.priority ASC NULLS LAST,
                    (regexp_match(o.district, '^[0-9]+'))[1]::int ASC NULLS LAST,
                    COALESCE(o.district, '') ASC,
                    (regexp_match(o.seat, '^[0-9]+'))[1]::int ASC NULLS LAST,
                    COALESCE(o.seat, '') ASC,
                    r.title DESC,
                    r.id ASC
                LIMIT $2 OFFSET $3
            )
            SELECT
                sr.id AS race_id, sr.slug AS race_slug, sr.title AS race_title,
                sr.total_votes, sr.num_precincts_reporting, sr.total_precincts,
                sr.winner_ids, sr.updated_at,
                sr.office_id, sr.office_title, sr.office_subtitle, sr.office_state,
                sr.office_county, sr.office_municipality, sr.office_district,
                sr.office_seat, sr.election_scope, sr.district_type,
                sr.political_scope,
                candidate.id AS candidate_id, candidate.slug AS candidate_slug,
                candidate.first_name, candidate.middle_name, candidate.last_name,
                candidate.suffix, candidate.preferred_name, rc.votes,
                party.id AS party_id, party.slug AS party_slug,
                party.name AS party_name, party.fec_code AS party_fec_code
            FROM selected_races sr
            LEFT JOIN race_candidates rc
              ON rc.race_id = sr.id AND rc.is_running = TRUE
            LEFT JOIN politician candidate ON candidate.id = rc.candidate_id
            LEFT JOIN party ON party.id = candidate.party_id
            ORDER BY
                sr.priority ASC NULLS LAST,
                (regexp_match(sr.office_district, '^[0-9]+'))[1]::int ASC NULLS LAST,
                COALESCE(sr.office_district, '') ASC,
                (regexp_match(sr.office_seat, '^[0-9]+'))[1]::int ASC NULLS LAST,
                COALESCE(sr.office_seat, '') ASC,
                sr.title DESC,
                sr.id ASC,
                candidate.last_name ASC NULLS LAST,
                candidate.first_name ASC NULLS LAST,
                candidate.id ASC NULLS LAST
            "#,
        )
        .bind(election_id)
        .bind(page.limit as i64)
        .bind(page.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(log_internal)?;
        Ok(Collection::new(group_results(rows), total, page))
    }

    async fn ballot_measures(
        &self,
        election_id: Uuid,
        filters: BallotMeasureFilters,
        page: PageRequest,
    ) -> Result<Collection<BallotMeasureResource>, ElectionDataError> {
        self.require_election(election_id).await?;
        let mut count = QueryBuilder::<Postgres>::new(
            "SELECT COUNT(*)::bigint FROM ballot_measure bm WHERE bm.election_id = ",
        );
        count.push_bind(election_id);
        apply_ballot_measure_filters(&mut count, &filters);
        let total = count
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(log_internal)?
            .max(0) as usize;

        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT bm.* FROM ballot_measure bm WHERE bm.election_id = ",
        );
        query.push_bind(election_id);
        apply_ballot_measure_filters(&mut query, &filters);
        query.push(" ORDER BY bm.title ASC, bm.id ASC LIMIT ");
        query.push_bind(page.limit as i64);
        query.push(" OFFSET ");
        query.push_bind(page.offset as i64);
        let data = query
            .build_query_as::<BallotMeasure>()
            .fetch_all(&self.pool)
            .await
            .map_err(log_internal)?
            .into_iter()
            .map(BallotMeasureResource::from)
            .collect();
        Ok(Collection::new(data, total, page))
    }
}

#[derive(sqlx::FromRow)]
struct RaceDirectoryRow {
    id: Uuid,
    slug: String,
    title: String,
    election_id: Option<Uuid>,
    race_type: RaceType,
    vote_type: VoteType,
    state: Option<UsState>,
    is_special_election: bool,
    num_elect: Option<i32>,
    total_votes: Option<i32>,
    num_precincts_reporting: Option<i32>,
    total_precincts: Option<i32>,
    winner_ids: Option<Vec<Uuid>>,
    updated_at: chrono::DateTime<chrono::Utc>,
    office_id: Uuid,
    office_title: String,
    office_subtitle: Option<String>,
    office_state: Option<UsState>,
    office_county: Option<String>,
    office_municipality: Option<String>,
    office_district: Option<String>,
    office_seat: Option<String>,
    election_scope: ElectionScope,
    district_type: Option<DistrictType>,
    political_scope: PoliticalScope,
    party_id: Option<Uuid>,
    party_slug: Option<String>,
    party_name: Option<String>,
    party_fec_code: Option<String>,
}

impl From<RaceDirectoryRow> for RaceSummaryResource {
    fn from(row: RaceDirectoryRow) -> Self {
        Self {
            id: row.id.to_string(),
            slug: row.slug,
            title: row.title,
            election_id: row.election_id.unwrap_or_default().to_string(),
            race_type: row.race_type,
            vote_type: row.vote_type,
            state: row.state,
            is_special_election: row.is_special_election,
            num_elect: row.num_elect,
            office: office_resource(
                row.office_id,
                row.office_title,
                row.office_subtitle,
                row.office_state,
                row.office_county,
                row.office_municipality,
                row.office_district,
                row.office_seat,
                row.election_scope,
                row.district_type,
                row.political_scope,
            ),
            party: party_resource(
                row.party_id,
                row.party_slug,
                row.party_name,
                row.party_fec_code,
            ),
            results: ResultSummaryResource {
                total_votes: row.total_votes,
                num_precincts_reporting: row.num_precincts_reporting,
                total_precincts: row.total_precincts,
                precinct_reporting_percentage: percentage(
                    row.num_precincts_reporting,
                    row.total_precincts,
                ),
                winners: ids(row.winner_ids),
            },
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ResultsRow {
    race_id: Uuid,
    race_slug: String,
    race_title: String,
    total_votes: Option<i32>,
    num_precincts_reporting: Option<i32>,
    total_precincts: Option<i32>,
    winner_ids: Option<Vec<Uuid>>,
    updated_at: chrono::DateTime<chrono::Utc>,
    office_id: Uuid,
    office_title: String,
    office_subtitle: Option<String>,
    office_state: Option<UsState>,
    office_county: Option<String>,
    office_municipality: Option<String>,
    office_district: Option<String>,
    office_seat: Option<String>,
    election_scope: ElectionScope,
    district_type: Option<DistrictType>,
    political_scope: PoliticalScope,
    candidate_id: Option<Uuid>,
    candidate_slug: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    suffix: Option<String>,
    preferred_name: Option<String>,
    votes: Option<i32>,
    party_id: Option<Uuid>,
    party_slug: Option<String>,
    party_name: Option<String>,
    party_fec_code: Option<String>,
}

fn group_results(rows: Vec<ResultsRow>) -> Vec<RaceResultsFeedResource> {
    let mut results: Vec<RaceResultsFeedResource> = Vec::new();
    for row in rows {
        let is_new_race = results
            .last()
            .map(|race| race.race_id != row.race_id.to_string())
            .unwrap_or(true);
        if is_new_race {
            results.push(RaceResultsFeedResource {
                race_id: row.race_id.to_string(),
                slug: row.race_slug.clone(),
                title: row.race_title.clone(),
                office: office_resource(
                    row.office_id,
                    row.office_title.clone(),
                    row.office_subtitle.clone(),
                    row.office_state,
                    row.office_county.clone(),
                    row.office_municipality.clone(),
                    row.office_district.clone(),
                    row.office_seat.clone(),
                    row.election_scope,
                    row.district_type,
                    row.political_scope,
                ),
                total_votes: row.total_votes,
                num_precincts_reporting: row.num_precincts_reporting,
                total_precincts: row.total_precincts,
                precinct_reporting_percentage: percentage(
                    row.num_precincts_reporting,
                    row.total_precincts,
                ),
                candidates: Vec::new(),
                winners: ids(row.winner_ids.clone()),
                updated_at: row.updated_at,
            });
        }
        if let Some(candidate_id) = row.candidate_id {
            let full_name = full_name(
                row.preferred_name.as_deref().or(row.first_name.as_deref()),
                row.middle_name.as_deref(),
                row.last_name.as_deref(),
                row.suffix.as_deref(),
            );
            let total_votes = row.total_votes;
            results
                .last_mut()
                .expect("a result was inserted for this row")
                .candidates
                .push(CandidateResultResource {
                    id: candidate_id.to_string(),
                    slug: row.candidate_slug.unwrap_or_default(),
                    full_name,
                    party: party_resource(
                        row.party_id,
                        row.party_slug,
                        row.party_name,
                        row.party_fec_code,
                    ),
                    votes: row.votes,
                    vote_percentage: percentage(row.votes, total_votes),
                });
        }
    }
    results
}

fn apply_election_filters(builder: &mut QueryBuilder<Postgres>, filters: &ElectionFilters) {
    if let Some(state) = filters.state {
        builder.push(" AND e.state = ");
        builder.push_bind(state);
    }
    if let Some(year) = filters.year {
        builder.push(" AND EXTRACT(YEAR FROM e.election_date)::integer = ");
        builder.push_bind(year);
    }
    if let Some(query) = filters.query.as_deref() {
        builder.push(
            " AND LOWER(e.title || ' ' || COALESCE(e.description, '') || ' ' || COALESCE(e.municipality, '')) LIKE ",
        );
        builder.push_bind(format!("%{}%", query.to_lowercase()));
    }
}

fn apply_race_filters(builder: &mut QueryBuilder<Postgres>, filters: &RaceFilters) {
    if let Some(state) = filters.state {
        builder.push(" AND r.state = ");
        builder.push_bind(state);
    }
    if let Some(race_type) = filters.race_type {
        builder.push(" AND r.race_type = ");
        builder.push_bind(race_type);
    }
    if let Some(scope) = filters.political_scope {
        builder.push(" AND o.political_scope = ");
        builder.push_bind(scope);
    }
    if let Some(scope) = filters.election_scope {
        builder.push(" AND o.election_scope = ");
        builder.push_bind(scope);
    }
    if let Some(district_type) = filters.district_type {
        builder.push(" AND o.district_type = ");
        builder.push_bind(district_type);
    }
    if let Some(query) = filters.query.as_deref() {
        builder.push(" AND LOWER(r.title || ' ' || o.title) LIKE ");
        builder.push_bind(format!("%{}%", query.to_lowercase()));
    }
}

fn apply_ballot_measure_filters(
    builder: &mut QueryBuilder<Postgres>,
    filters: &BallotMeasureFilters,
) {
    if let Some(state) = filters.state {
        builder.push(" AND bm.state = ");
        builder.push_bind(state);
    }
    if let Some(status) = filters.status {
        builder.push(" AND bm.status::text = ");
        builder.push_bind(status);
    }
    if let Some(scope) = filters.election_scope {
        builder.push(" AND bm.election_scope = ");
        builder.push_bind(scope);
    }
    if let Some(county) = filters.county.as_deref() {
        builder.push(" AND LOWER(bm.county) = ");
        builder.push_bind(county.to_lowercase());
    }
    if let Some(municipality) = filters.municipality.as_deref() {
        builder.push(" AND LOWER(bm.municipality) = ");
        builder.push_bind(municipality.to_lowercase());
    }
    if let Some(school_district) = filters.school_district.as_deref() {
        builder.push(" AND LOWER(bm.school_district) = ");
        builder.push_bind(school_district.to_lowercase());
    }
}

fn push_race_order(builder: &mut QueryBuilder<Postgres>) {
    builder.push(
        " ORDER BY o.priority ASC NULLS LAST, \
         (regexp_match(o.district, '^[0-9]+'))[1]::int ASC NULLS LAST, \
         COALESCE(o.district, '') ASC, \
         (regexp_match(o.seat, '^[0-9]+'))[1]::int ASC NULLS LAST, \
         COALESCE(o.seat, '') ASC, r.title DESC, r.id ASC",
    );
}

fn office_resource(
    id: Uuid,
    title: String,
    subtitle: Option<String>,
    state: Option<UsState>,
    county: Option<String>,
    municipality: Option<String>,
    district: Option<String>,
    seat: Option<String>,
    election_scope: ElectionScope,
    district_type: Option<DistrictType>,
    political_scope: PoliticalScope,
) -> OfficeSummaryResource {
    OfficeSummaryResource {
        id: id.to_string(),
        title,
        subtitle,
        state,
        county,
        municipality,
        district,
        seat,
        election_scope,
        district_type,
        political_scope,
    }
}

fn party_resource(
    id: Option<Uuid>,
    slug: Option<String>,
    name: Option<String>,
    fec_code: Option<String>,
) -> Option<PartySummaryResource> {
    Some(PartySummaryResource {
        id: id?.to_string(),
        slug: slug?,
        name: name?,
        fec_code,
    })
}

fn ids(values: Option<Vec<Uuid>>) -> Vec<IdResource> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(|id| IdResource { id: id.to_string() })
        .collect()
}

fn full_name(
    first: Option<&str>,
    middle: Option<&str>,
    last: Option<&str>,
    suffix: Option<&str>,
) -> String {
    [first, middle, last, suffix]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn percentage(numerator: Option<i32>, denominator: Option<i32>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if denominator > 0 => {
            Some(((f64::from(numerator) / f64::from(denominator) * 100.0) * 10.0).round() / 10.0)
        }
        _ => None,
    }
}

fn ballot_measure_status_name(status: db::BallotMeasureStatus) -> &'static str {
    match status {
        db::BallotMeasureStatus::Introduced => "introduced",
        db::BallotMeasureStatus::InConsideration => "in_consideration",
        db::BallotMeasureStatus::Proposed => "proposed",
        db::BallotMeasureStatus::GatheringSignatures => "gathering_signatures",
        db::BallotMeasureStatus::OnTheBallot => "on_the_ballot",
        db::BallotMeasureStatus::BecameLaw => "became_law",
        db::BallotMeasureStatus::Failed => "failed",
        db::BallotMeasureStatus::Unknown => "unknown",
    }
}

fn parse_ballot_measure_status(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "introduced" => Some("introduced"),
        "in_consideration" => Some("in_consideration"),
        "proposed" => Some("proposed"),
        "gathering_signatures" => Some("gathering_signatures"),
        "on_the_ballot" => Some("on_the_ballot"),
        "became_law" => Some("became_law"),
        "failed" => Some("failed"),
        "unknown" => Some("unknown"),
        _ => None,
    }
}

fn map_election_error(error: sqlx::Error) -> ElectionDataError {
    match error {
        sqlx::Error::RowNotFound => ElectionDataError::ElectionNotFound,
        error => log_internal(error),
    }
}

fn map_race_error(error: sqlx::Error) -> ElectionDataError {
    match error {
        sqlx::Error::RowNotFound => ElectionDataError::RaceNotFound,
        error => log_internal(error),
    }
}

fn log_internal(error: sqlx::Error) -> ElectionDataError {
    #[cfg(test)]
    eprintln!("REST election data lookup failed: {error:?}");
    tracing::error!(error = ?error, "REST election data lookup failed");
    ElectionDataError::Internal
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ElectionsQuery {
    state: Option<String>,
    year: Option<String>,
    query: Option<String>,
    limit: Option<String>,
    offset: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RacesQuery {
    state: Option<String>,
    race_type: Option<String>,
    political_scope: Option<String>,
    election_scope: Option<String>,
    district_type: Option<String>,
    query: Option<String>,
    limit: Option<String>,
    offset: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RaceQuery {
    endorser_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PageQuery {
    limit: Option<String>,
    offset: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BallotMeasuresQuery {
    state: Option<String>,
    status: Option<String>,
    election_scope: Option<String>,
    county: Option<String>,
    municipality: Option<String>,
    school_district: Option<String>,
    limit: Option<String>,
    offset: Option<String>,
}

pub(super) async fn list(
    State(state): State<RestState>,
    query: Result<Query<ElectionsQuery>, QueryRejection>,
) -> Result<(HeaderMap, Json<Collection<ElectionResource>>), ApiError> {
    let Query(query) = parse_query(query, ELECTIONS_ENDPOINT)?;
    let page = parse_page(query.limit, query.offset, ELECTIONS_ENDPOINT)?;
    let year = parse_optional_i32(query.year, "year", ELECTIONS_ENDPOINT)?;
    if year.is_some_and(|year| !(1788..=9999).contains(&year)) {
        return Err(ApiError::invalid_parameter(
            "year",
            "must be between 1788 and 9999",
            ELECTIONS_ENDPOINT,
        ));
    }
    let filters = ElectionFilters {
        state: parse_optional_state(query.state, ELECTIONS_ENDPOINT)?,
        year,
        query: clean_optional(query.query),
    };
    let data = state
        .elections
        .elections(filters, page)
        .await
        .map_err(|error| map_lookup_error(error, None, None, ELECTIONS_ENDPOINT))?;
    Ok(cached(data, CACHE_METADATA))
}

pub(super) async fn get_one(
    State(state): State<RestState>,
    Path(election_id): Path<String>,
) -> Result<(HeaderMap, Json<Resource<ElectionResource>>), ApiError> {
    let instance = format!("/api/v1/elections/{election_id}");
    let election_id = parse_id(&election_id, "electionId", &instance)?;
    let data = state
        .elections
        .election(election_id)
        .await
        .map_err(|error| map_lookup_error(error, Some(election_id), None, &instance))?;
    Ok(cached(Resource { data }, CACHE_METADATA))
}

pub(super) async fn list_races(
    State(state): State<RestState>,
    Path(election_id): Path<String>,
    query: Result<Query<RacesQuery>, QueryRejection>,
) -> Result<(HeaderMap, Json<Collection<RaceSummaryResource>>), ApiError> {
    let instance = format!("/api/v1/elections/{election_id}/races");
    let election_id = parse_id(&election_id, "electionId", &instance)?;
    let Query(query) = parse_query(query, &instance)?;
    let page = parse_page(query.limit, query.offset, &instance)?;
    let filters = RaceFilters {
        state: parse_optional_state(query.state, &instance)?,
        race_type: parse_optional_api_enum(query.race_type, "raceType", &instance)?,
        political_scope: parse_optional_api_enum(
            query.political_scope,
            "politicalScope",
            &instance,
        )?,
        election_scope: parse_optional_api_enum(query.election_scope, "electionScope", &instance)?,
        district_type: parse_optional_api_enum(query.district_type, "districtType", &instance)?,
        query: clean_optional(query.query),
    };
    let data = state
        .elections
        .races(election_id, filters, page)
        .await
        .map_err(|error| map_lookup_error(error, Some(election_id), None, &instance))?;
    Ok(cached(data, CACHE_COLLECTION))
}

pub(super) async fn get_race(
    State(state): State<RestState>,
    Path((election_id, race_id)): Path<(String, String)>,
    query: Result<Query<RaceQuery>, QueryRejection>,
) -> Result<(HeaderMap, Json<Resource<RaceDetailResource>>), ApiError> {
    let instance = format!("/api/v1/elections/{election_id}/races/{race_id}");
    let election_id = parse_id(&election_id, "electionId", &instance)?;
    let race_id = parse_id(&race_id, "raceId", &instance)?;
    let Query(query) = parse_query(query, &instance)?;
    let endorser_id = query
        .endorser_id
        .map(|value| {
            Uuid::parse_str(&value).map_err(|_| {
                ApiError::invalid_parameter("endorserId", "must be a UUID", instance.clone())
            })
        })
        .transpose()?;
    let data = state
        .elections
        .race(election_id, race_id, endorser_id)
        .await
        .map_err(|error| map_lookup_error(error, Some(election_id), Some(race_id), &instance))?;
    Ok(cached(Resource { data }, CACHE_COLLECTION))
}

pub(super) async fn list_results(
    State(state): State<RestState>,
    Path(election_id): Path<String>,
    query: Result<Query<PageQuery>, QueryRejection>,
) -> Result<(HeaderMap, Json<Collection<RaceResultsFeedResource>>), ApiError> {
    let instance = format!("/api/v1/elections/{election_id}/results");
    let election_id = parse_id(&election_id, "electionId", &instance)?;
    let Query(query) = parse_query(query, &instance)?;
    let page = parse_page(query.limit, query.offset, &instance)?;
    let data = state
        .elections
        .results(election_id, page)
        .await
        .map_err(|error| map_lookup_error(error, Some(election_id), None, &instance))?;
    Ok(cached(data, CACHE_RESULTS))
}

pub(super) async fn list_ballot_measures(
    State(state): State<RestState>,
    Path(election_id): Path<String>,
    query: Result<Query<BallotMeasuresQuery>, QueryRejection>,
) -> Result<(HeaderMap, Json<Collection<BallotMeasureResource>>), ApiError> {
    let instance = format!("/api/v1/elections/{election_id}/ballot-measures");
    let election_id = parse_id(&election_id, "electionId", &instance)?;
    let Query(query) = parse_query(query, &instance)?;
    let page = parse_page(query.limit, query.offset, &instance)?;
    let status = query
        .status
        .map(|value| {
            parse_ballot_measure_status(&value).ok_or_else(|| {
                ApiError::invalid_parameter(
                    "status",
                    "must be a valid ballot-measure status",
                    instance.clone(),
                )
            })
        })
        .transpose()?;
    let filters = BallotMeasureFilters {
        state: parse_optional_state(query.state, &instance)?,
        status,
        election_scope: parse_optional_api_enum(query.election_scope, "electionScope", &instance)?,
        county: clean_optional(query.county),
        municipality: clean_optional(query.municipality),
        school_district: clean_optional(query.school_district),
    };
    let data = state
        .elections
        .ballot_measures(election_id, filters, page)
        .await
        .map_err(|error| map_lookup_error(error, Some(election_id), None, &instance))?;
    Ok(cached(data, CACHE_COLLECTION))
}

fn parse_query<T>(
    query: Result<Query<T>, QueryRejection>,
    instance: &str,
) -> Result<Query<T>, ApiError> {
    query.map_err(|error| {
        ApiError::malformed_query(
            format!("The query string could not be parsed: {error}"),
            instance,
        )
    })
}

fn parse_page(
    limit: Option<String>,
    offset: Option<String>,
    instance: &str,
) -> Result<PageRequest, ApiError> {
    let limit = parse_usize(limit, "limit", DEFAULT_LIMIT, instance)?;
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(ApiError::invalid_parameter(
            "limit",
            format!("must be between 1 and {MAX_LIMIT}"),
            instance,
        ));
    }
    let offset = parse_usize(offset, "offset", 0, instance)?;
    if offset > i64::MAX as usize {
        return Err(ApiError::invalid_parameter(
            "offset",
            "is too large",
            instance,
        ));
    }
    Ok(PageRequest { limit, offset })
}

fn parse_usize(
    value: Option<String>,
    name: &'static str,
    default: usize,
    instance: &str,
) -> Result<usize, ApiError> {
    value
        .map(|value| {
            value.parse::<usize>().map_err(|_| {
                ApiError::invalid_parameter(name, "must be a non-negative integer", instance)
            })
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn parse_optional_i32(
    value: Option<String>,
    name: &'static str,
    instance: &str,
) -> Result<Option<i32>, ApiError> {
    value
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|_| ApiError::invalid_parameter(name, "must be an integer", instance))
        })
        .transpose()
}

fn parse_optional_state(
    value: Option<String>,
    instance: &str,
) -> Result<Option<UsState>, ApiError> {
    value
        .map(|value| {
            serde_json::from_value(serde_json::Value::String(value.trim().to_ascii_uppercase()))
                .map_err(|_| {
                    ApiError::invalid_parameter(
                        "state",
                        "must be a two-letter US state or territory code",
                        instance,
                    )
                })
        })
        .transpose()
}

fn parse_optional_api_enum<T>(
    value: Option<String>,
    name: &'static str,
    instance: &str,
) -> Result<Option<T>, ApiError>
where
    T: DeserializeOwned,
{
    value
        .map(|value| {
            serde_json::from_value(serde_json::Value::String(value.trim().to_ascii_lowercase()))
                .map_err(|_| {
                    ApiError::invalid_parameter(name, "has an unsupported value", instance)
                })
        })
        .transpose()
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_id(value: &str, parameter: &'static str, instance: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value)
        .map_err(|_| ApiError::invalid_path_parameter(parameter, "must be a UUID", instance))
}

fn map_lookup_error(
    error: ElectionDataError,
    election_id: Option<Uuid>,
    race_id: Option<Uuid>,
    instance: &str,
) -> ApiError {
    match error {
        ElectionDataError::ElectionNotFound => ApiError::resource_not_found(
            "election",
            election_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            instance,
        ),
        ElectionDataError::RaceNotFound => ApiError::resource_not_found(
            "race",
            race_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            instance,
        ),
        ElectionDataError::Internal => ApiError::internal(instance),
    }
}

fn cached<T>(data: T, cache_control: HeaderValue) -> (HeaderMap, Json<T>) {
    let mut headers = HeaderMap::new();
    headers.insert(CACHE_CONTROL, cache_control);
    (headers, Json(data))
}

#[cfg(test)]
pub(super) struct UnavailableElectionDataLookup;

#[cfg(test)]
#[async_trait]
impl ElectionDataLookup for UnavailableElectionDataLookup {
    async fn elections(
        &self,
        _filters: ElectionFilters,
        _page: PageRequest,
    ) -> Result<Collection<ElectionResource>, ElectionDataError> {
        Err(ElectionDataError::Internal)
    }

    async fn election(&self, _election_id: Uuid) -> Result<ElectionResource, ElectionDataError> {
        Err(ElectionDataError::Internal)
    }

    async fn races(
        &self,
        _election_id: Uuid,
        _filters: RaceFilters,
        _page: PageRequest,
    ) -> Result<Collection<RaceSummaryResource>, ElectionDataError> {
        Err(ElectionDataError::Internal)
    }

    async fn race(
        &self,
        _election_id: Uuid,
        _race_id: Uuid,
        _endorser_id: Option<Uuid>,
    ) -> Result<RaceDetailResource, ElectionDataError> {
        Err(ElectionDataError::Internal)
    }

    async fn results(
        &self,
        _election_id: Uuid,
        _page: PageRequest,
    ) -> Result<Collection<RaceResultsFeedResource>, ElectionDataError> {
        Err(ElectionDataError::Internal)
    }

    async fn ballot_measures(
        &self,
        _election_id: Uuid,
        _filters: BallotMeasureFilters,
        _page: PageRequest,
    ) -> Result<Collection<BallotMeasureResource>, ElectionDataError> {
        Err(ElectionDataError::Internal)
    }
}

pub(super) fn endpoint_templates() -> [&'static str; 6] {
    [
        "/api/v1/elections",
        "/api/v1/elections/{electionId}",
        "/api/v1/elections/{electionId}/races",
        "/api/v1/elections/{electionId}/races/{raceId}",
        "/api/v1/elections/{electionId}/results",
        "/api/v1/elections/{electionId}/ballot-measures",
    ]
}

pub(super) type SharedElectionDataLookup = Arc<dyn ElectionDataLookup>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and reads the configured database"]
    async fn election_data_queries_decode_the_live_database_schema() {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("database must be reachable");
        let election_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT e.id
            FROM election e
            WHERE EXISTS (SELECT 1 FROM race r WHERE r.election_id = e.id)
            ORDER BY e.election_date DESC, e.id
            LIMIT 1
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("an election with races must exist");
        let lookup = DatabaseElectionDataLookup::new(pool);
        let page = PageRequest {
            limit: 10,
            offset: 0,
        };

        lookup
            .elections(ElectionFilters::default(), page)
            .await
            .expect("election collection should decode");
        lookup
            .election(election_id)
            .await
            .expect("election detail should decode");
        let races = lookup
            .races(election_id, RaceFilters::default(), page)
            .await
            .expect("race collection should decode");
        lookup
            .results(election_id, page)
            .await
            .expect("results feed should decode");
        lookup
            .ballot_measures(election_id, BallotMeasureFilters::default(), page)
            .await
            .expect("ballot-measure collection should decode");
        if let Some(race) = races.data.first() {
            lookup
                .race(
                    election_id,
                    Uuid::parse_str(&race.id).expect("race ID should be a UUID"),
                    None,
                )
                .await
                .expect("race detail should decode");
        }
    }
}
