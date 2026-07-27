use async_trait::async_trait;
use axum::{
    extract::{
        rejection::{JsonRejection, QueryRejection},
        Path, Query, State,
    },
    http::{
        header::{CACHE_CONTROL, PRAGMA},
        HeaderMap, HeaderValue,
    },
    Json,
};
use db::{
    AddressInput, BallotMeasure, DistrictType, Election, ElectionScope, Office, Party,
    PoliticalScope, Politician, Race, State as UsState,
};
use graphql::types::{
    get_ballot_measures_by_address_id, get_races_by_address_id,
    process_address_with_geocodio_status,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, QueryBuilder};
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use uuid::Uuid;

use super::{error::ApiError, RestState};

const ENDPOINT_TEMPLATE: &str = "/api/v1/elections/{electionId}/ballot";
#[cfg(not(test))]
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(20);
#[cfg(test)]
const LOOKUP_TIMEOUT: Duration = Duration::from_millis(25);
const MAX_CONCURRENT_LOOKUPS: usize = 32;
pub(super) const MAX_REQUEST_BODY_BYTES: usize = 8 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BallotRequest {
    address: AddressRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AddressRequest {
    line_1: String,
    line_2: Option<String>,
    city: String,
    state: String,
    postal_code: String,
    country: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BallotQuery {
    endorser_id: Option<String>,
}

#[derive(Debug)]
pub(super) struct BallotLookupRequest {
    pub(super) election_id: Uuid,
    pub(super) address: AddressInput,
    pub(super) endorser_id: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub(super) enum BallotLookupError {
    #[error("election not found")]
    ElectionNotFound,
    #[error("address could not be resolved")]
    InvalidAddress,
    #[error("address service unavailable")]
    AddressServiceUnavailable,
    #[error("ballot lookup capacity exhausted")]
    RateLimited,
    #[error("ballot lookup failed")]
    Internal,
}

#[async_trait]
pub(super) trait BallotLookup: Send + Sync {
    async fn lookup(&self, request: BallotLookupRequest) -> Result<BallotData, BallotLookupError>;
}

pub(super) struct DatabaseBallotLookup {
    pool: PgPool,
    permits: Arc<Semaphore>,
}

impl DatabaseBallotLookup {
    pub(super) fn new(pool: PgPool) -> Self {
        Self {
            pool,
            permits: Arc::new(Semaphore::new(MAX_CONCURRENT_LOOKUPS)),
        }
    }

    async fn race_resource(
        &self,
        race: Race,
        election_date: chrono::NaiveDate,
        endorser_id: Option<Uuid>,
    ) -> Result<RaceResource, sqlx::Error> {
        let (office, party, incumbents, candidates, votes_by_candidate, related_embeds) = tokio::try_join!(
            Office::find_by_id(&self.pool, race.office_id),
            party_by_id(&self.pool, race.party_id),
            incumbents_for_office(&self.pool, race.office_id),
            candidates_for_race(&self.pool, race.id, endorser_id),
            votes_for_race(&self.pool, race.id, race.total_votes),
            related_embeds_for_race(&self.pool, &race),
        )?;

        Ok(RaceResource {
            id: race.id.to_string(),
            slug: race.slug,
            title: race.title,
            election_id: race.election_id.map(|id| id.to_string()),
            office_id: race.office_id.to_string(),
            race_type: race.race_type,
            vote_type: race.vote_type,
            state: race.state,
            election_date,
            is_special_election: race.is_special_election,
            num_elect: race.num_elect,
            party,
            office: OfficeResource::new(office, incumbents),
            candidates,
            results: RaceResultsResource {
                total_votes: race.total_votes,
                precinct_reporting_percentage: percentage(
                    race.num_precincts_reporting,
                    race.total_precincts,
                ),
                votes_by_candidate,
                winners: race
                    .winner_ids
                    .unwrap_or_default()
                    .into_iter()
                    .map(|id| IdResource { id: id.to_string() })
                    .collect(),
            },
            related_embeds,
        })
    }
}

#[async_trait]
impl BallotLookup for DatabaseBallotLookup {
    async fn lookup(&self, request: BallotLookupRequest) -> Result<BallotData, BallotLookupError> {
        let _permit = self
            .permits
            .try_acquire()
            .map_err(|_| BallotLookupError::RateLimited)?;
        let coverage = CoverageResource::for_state(request.address.state);
        let election = Election::find_by_id(&self.pool, request.election_id)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => BallotLookupError::ElectionNotFound,
                _ => {
                    tracing::error!(
                        error = ?error,
                        election_id = %request.election_id,
                        "failed to load election for REST ballot lookup"
                    );
                    BallotLookupError::Internal
                }
            })?;

        let processed_address = process_address_with_geocodio_status(&self.pool, request.address)
            .await
            .map_err(|error| {
                let mapped = match &error {
                    graphql::types::Error::BadInput { .. } | graphql::types::Error::BadAddress => {
                        BallotLookupError::InvalidAddress
                    }
                    graphql::types::Error::GeocodioError(_)
                    | graphql::types::Error::VarError(_) => {
                        BallotLookupError::AddressServiceUnavailable
                    }
                    _ => BallotLookupError::Internal,
                };
                tracing::warn!(
                    error = ?error,
                    election_id = %request.election_id,
                    "REST ballot address resolution failed"
                );
                mapped
            })?;

        let result = async {
            let (races, ballot_measures) = tokio::try_join!(
                get_races_by_address_id(&self.pool, &request.election_id, &processed_address.id),
                get_ballot_measures_by_address_id(
                    &self.pool,
                    &request.election_id,
                    &processed_address.id
                ),
            )
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    election_id = %request.election_id,
                    "REST ballot selection failed"
                );
                BallotLookupError::Internal
            })?;

            let mut race_resources = Vec::with_capacity(races.len());
            for race in races {
                race_resources.push(
                    self.race_resource(race, election.election_date, request.endorser_id)
                        .await
                        .map_err(|error| {
                            tracing::error!(
                                error = ?error,
                                election_id = %request.election_id,
                                "REST ballot response enrichment failed"
                            );
                            BallotLookupError::Internal
                        })?,
                );
            }

            Ok(BallotData {
                election: ElectionResource {
                    id: election.id.to_string(),
                    slug: election.slug,
                    title: election.title,
                    description: election.description,
                    state: election.state,
                    election_date: election.election_date,
                },
                races: race_resources,
                ballot_measures: ballot_measures
                    .into_iter()
                    .map(BallotMeasureResource::from)
                    .collect(),
                coverage,
            })
        }
        .await;

        if processed_address.created {
            if let Err(error) = sqlx::query(
                r#"
                DELETE FROM address
                WHERE id = $1
                  AND NOT EXISTS (
                      SELECT 1 FROM user_profile WHERE address_id = $1
                  )
                "#,
            )
            .bind(processed_address.id)
            .execute(&self.pool)
            .await
            {
                tracing::error!(
                    error = ?error,
                    election_id = %request.election_id,
                    "failed to remove temporary REST ballot address"
                );
            }
        }

        result
    }
}

#[derive(Debug, Serialize)]
pub(super) struct BallotResponse {
    data: BallotData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BallotData {
    election: ElectionResource,
    races: Vec<RaceResource>,
    ballot_measures: Vec<BallotMeasureResource>,
    coverage: CoverageResource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CoverageResource {
    races: &'static str,
    ballot_measures: &'static str,
    warnings: Vec<&'static str>,
}

impl CoverageResource {
    fn for_state(state: UsState) -> Self {
        match state {
            UsState::MN => Self {
                races: "address_specific",
                ballot_measures: "address_specific",
                warnings: Vec::new(),
            },
            UsState::TX => Self {
                races: "address_specific",
                ballot_measures: "statewide_only",
                warnings: vec![
                    "Local ballot-measure matching is not currently available for Texas.",
                ],
            },
            _ => Self {
                races: "statewide_only",
                ballot_measures: "statewide_only",
                warnings: vec![
                    "District and local contest matching is not currently available for this state.",
                ],
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ElectionResource {
    id: String,
    slug: String,
    title: String,
    description: Option<String>,
    state: Option<UsState>,
    election_date: chrono::NaiveDate,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RaceResource {
    id: String,
    slug: String,
    title: String,
    election_id: Option<String>,
    office_id: String,
    race_type: db::RaceType,
    vote_type: db::VoteType,
    state: Option<UsState>,
    election_date: chrono::NaiveDate,
    is_special_election: bool,
    num_elect: Option<i32>,
    party: Option<PartyResource>,
    office: OfficeResource,
    candidates: Vec<PoliticianResource>,
    results: RaceResultsResource,
    related_embeds: Vec<EmbedResource>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OfficeResource {
    id: String,
    name: Option<String>,
    title: String,
    subtitle: Option<String>,
    subtitle_short: Option<String>,
    state: Option<UsState>,
    county: Option<String>,
    municipality: Option<String>,
    district: Option<String>,
    seat: Option<String>,
    election_scope: ElectionScope,
    district_type: Option<DistrictType>,
    political_scope: PoliticalScope,
    incumbents: Vec<IncumbentResource>,
}

impl OfficeResource {
    fn new(office: Office, incumbents: Vec<IncumbentResource>) -> Self {
        let subtitle = office
            .subtitle
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| computed_office_subtitle(&office, false));
        let subtitle_short = office
            .subtitle_short
            .clone()
            .or_else(|| computed_office_subtitle(&office, true));

        Self {
            id: office.id.to_string(),
            name: office.name,
            title: office.title,
            subtitle,
            subtitle_short,
            state: office.state,
            county: office.county,
            municipality: office.municipality,
            district: office.district,
            seat: office.seat,
            election_scope: office.election_scope,
            district_type: office.district_type,
            political_scope: office.political_scope,
            incumbents,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PoliticianResource {
    id: String,
    slug: String,
    full_name: String,
    email: Option<String>,
    phone: Option<String>,
    party: Option<PartyResource>,
    thumbnail_image_url: Option<String>,
    assets: PoliticianAssetsResource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IncumbentResource {
    id: String,
    full_name: String,
    party: Option<PartyResource>,
    thumbnail_image_url: Option<String>,
    assets: PoliticianAssetsResource,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PoliticianAssetsResource {
    thumbnail_image_160: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PartyResource {
    name: String,
    fec_code: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RaceResultsResource {
    total_votes: Option<i32>,
    precinct_reporting_percentage: Option<f64>,
    votes_by_candidate: Vec<CandidateVotesResource>,
    winners: Vec<IdResource>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CandidateVotesResource {
    candidate_id: String,
    votes: Option<i32>,
    vote_percentage: Option<f64>,
}

#[derive(Debug, Serialize)]
struct IdResource {
    id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedResource {
    id: String,
    organization_id: String,
    embed_type: &'static str,
    race: Option<RaceReferenceResource>,
}

#[derive(Debug, Serialize)]
struct RaceReferenceResource {
    title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BallotMeasureResource {
    id: String,
    title: String,
    description: Option<String>,
    state: UsState,
    ballot_measure_code: String,
    yes_votes: Option<i32>,
    no_votes: Option<i32>,
    num_precincts_reporting: Option<i32>,
    total_precincts: Option<i32>,
    election_scope: Option<ElectionScope>,
}

impl From<BallotMeasure> for BallotMeasureResource {
    fn from(measure: BallotMeasure) -> Self {
        Self {
            id: measure.id.to_string(),
            title: measure.title,
            description: measure.description,
            state: measure.state,
            ballot_measure_code: measure.ballot_measure_code,
            yes_votes: measure.yes_votes,
            no_votes: measure.no_votes,
            num_precincts_reporting: measure.num_precincts_reporting,
            total_precincts: measure.total_precincts,
            election_scope: measure.election_scope,
        }
    }
}

pub(super) async fn lookup(
    State(state): State<RestState>,
    Path(election_id): Path<String>,
    query: Result<Query<BallotQuery>, QueryRejection>,
    payload: Result<Json<BallotRequest>, JsonRejection>,
) -> Result<(HeaderMap, Json<BallotResponse>), ApiError> {
    let instance = format!("/api/v1/elections/{election_id}/ballot");
    let election_id = Uuid::parse_str(&election_id).map_err(|_| {
        ApiError::invalid_path_parameter("electionId", "must be a UUID", instance.clone())
    })?;
    let Query(query) = query.map_err(|error| {
        ApiError::malformed_query(
            format!("The query string could not be parsed: {error}"),
            instance.clone(),
        )
    })?;
    let endorser_id = query
        .endorser_id
        .map(|value| {
            Uuid::parse_str(&value).map_err(|_| {
                ApiError::invalid_parameter("endorserId", "must be a UUID", instance.clone())
            })
        })
        .transpose()?;
    let Json(payload) = payload.map_err(|error| match error.status() {
        axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE => {
            ApiError::unsupported_media_type(instance.clone())
        }
        axum::http::StatusCode::PAYLOAD_TOO_LARGE => ApiError::payload_too_large(instance.clone()),
        _ => ApiError::invalid_body(
            format!("The JSON request body is invalid: {}", error.body_text()),
            instance.clone(),
        ),
    })?;
    let address = payload.address.validate(&instance)?;

    let data = tokio::time::timeout(
        LOOKUP_TIMEOUT,
        state.ballots.lookup(BallotLookupRequest {
            election_id,
            address,
            endorser_id,
        }),
    )
    .await
    .map_err(|_| ApiError::gateway_timeout(instance.clone()))?
    .map_err(|error| match error {
        BallotLookupError::ElectionNotFound => {
            ApiError::resource_not_found("election", election_id, instance.clone())
        }
        BallotLookupError::InvalidAddress => ApiError::unprocessable(
            "invalid_address",
            "The address could not be resolved to a voting location.",
            instance.clone(),
        ),
        BallotLookupError::AddressServiceUnavailable => ApiError::service_unavailable(
            "address_service_unavailable",
            "The address service is temporarily unavailable.",
            instance.clone(),
        ),
        BallotLookupError::RateLimited => ApiError::rate_limited(instance.clone()),
        BallotLookupError::Internal => ApiError::internal(instance.clone()),
    })?;

    let mut headers = HeaderMap::new();
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    Ok((headers, Json(BallotResponse { data })))
}

impl AddressRequest {
    fn validate(self, instance: &str) -> Result<AddressInput, ApiError> {
        let line_1 = required_text(self.line_1, "address.line1", 200, instance)?;
        let _line_2 = optional_text(self.line_2, "address.line2", 200, instance)?;
        let city = required_text(self.city, "address.city", 100, instance)?;
        let state_code = required_text(self.state, "address.state", 2, instance)?.to_uppercase();
        let state = UsState::from_str(&state_code).map_err(|_| {
            ApiError::unprocessable(
                "invalid_address",
                "`address.state` must be a valid two-letter US state or territory code.",
                instance,
            )
        })?;
        let postal_code = required_text(self.postal_code, "address.postalCode", 10, instance)?;
        if !valid_postal_code(&postal_code) {
            return Err(ApiError::unprocessable(
                "invalid_address",
                "`address.postalCode` must be a five-digit ZIP code with an optional four-digit extension.",
                instance,
            ));
        }
        let country = self
            .country
            .unwrap_or_else(|| "US".to_string())
            .trim()
            .to_uppercase();
        if country != "US" && country != "USA" {
            return Err(ApiError::unprocessable(
                "invalid_address",
                "`address.country` must be `US` or `USA`.",
                instance,
            ));
        }

        Ok(AddressInput {
            line_1,
            // Unit information is not needed to resolve voting districts and is deliberately
            // excluded from geocoding and temporary storage.
            line_2: None,
            city,
            county: None,
            state,
            country: "US".to_string(),
            postal_code,
            coordinates: None,
            congressional_district: None,
            state_senate_district: None,
            state_house_district: None,
        })
    }
}

fn required_text(
    value: String,
    field: &'static str,
    max_len: usize,
    instance: &str,
) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::unprocessable(
            "invalid_address",
            format!("`{field}` is required."),
            instance,
        ));
    }
    validate_text(value, field, max_len, instance)?;
    Ok(value.to_string())
}

fn optional_text(
    value: Option<String>,
    field: &'static str,
    max_len: usize,
    instance: &str,
) -> Result<Option<String>, ApiError> {
    value
        .map(|value| {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                validate_text(value, field, max_len, instance)?;
                Ok(Some(value.to_string()))
            }
        })
        .transpose()
        .map(Option::flatten)
}

fn validate_text(
    value: &str,
    field: &'static str,
    max_len: usize,
    instance: &str,
) -> Result<(), ApiError> {
    if value.chars().count() > max_len {
        return Err(ApiError::unprocessable(
            "invalid_address",
            format!("`{field}` must contain at most {max_len} characters."),
            instance,
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(ApiError::unprocessable(
            "invalid_address",
            format!("`{field}` must not contain control characters."),
            instance,
        ));
    }
    Ok(())
}

fn valid_postal_code(value: &str) -> bool {
    let bytes = value.as_bytes();
    (bytes.len() == 5 && bytes.iter().all(u8::is_ascii_digit))
        || (bytes.len() == 10
            && bytes[0..5].iter().all(u8::is_ascii_digit)
            && bytes[5] == b'-'
            && bytes[6..10].iter().all(u8::is_ascii_digit))
}

async fn party_by_id(
    pool: &PgPool,
    party_id: Option<Uuid>,
) -> Result<Option<PartyResource>, sqlx::Error> {
    let Some(party_id) = party_id else {
        return Ok(None);
    };
    sqlx::query_as::<_, Party>("SELECT * FROM party WHERE id = $1")
        .bind(party_id)
        .fetch_optional(pool)
        .await
        .map(|party| party.map(PartyResource::from))
}

impl From<Party> for PartyResource {
    fn from(party: Party) -> Self {
        Self {
            name: party.name,
            fec_code: party.fec_code,
        }
    }
}

async fn incumbents_for_office(
    pool: &PgPool,
    office_id: Uuid,
) -> Result<Vec<IncumbentResource>, sqlx::Error> {
    let politicians = sqlx::query_as::<_, Politician>(
        "SELECT * FROM politician WHERE office_id = $1 ORDER BY last_name, first_name, id",
    )
    .bind(office_id)
    .fetch_all(pool)
    .await?;
    let mut resources = Vec::with_capacity(politicians.len());
    for politician in politicians {
        let party = party_by_id(pool, politician.party_id).await?;
        resources.push(IncumbentResource {
            id: politician.id.to_string(),
            full_name: politician_full_name(&politician),
            party,
            thumbnail_image_url: politician.thumbnail_image_url,
            assets: serde_json::from_value(politician.assets).unwrap_or_default(),
        });
    }
    Ok(resources)
}

async fn candidates_for_race(
    pool: &PgPool,
    race_id: Uuid,
    endorser_id: Option<Uuid>,
) -> Result<Vec<PoliticianResource>, sqlx::Error> {
    let mut builder = QueryBuilder::new(
        "SELECT p.* FROM politician p JOIN race_candidates rc ON rc.candidate_id = p.id",
    );
    builder.push(" WHERE rc.race_id = ");
    builder.push_bind(race_id);
    builder.push(" AND rc.is_running = TRUE");
    if let Some(endorser_id) = endorser_id {
        builder.push(
            " AND EXISTS (
                SELECT 1
                FROM politician_organization_endorsements poe
                WHERE poe.politician_id = p.id
                  AND poe.organization_id = ",
        );
        builder.push_bind(endorser_id);
        builder.push(")");
    }
    builder.push(" ORDER BY p.last_name, p.first_name, p.id");

    let politicians = builder
        .build_query_as::<Politician>()
        .fetch_all(pool)
        .await?;
    politician_resources(pool, politicians).await
}

async fn politician_resources(
    pool: &PgPool,
    politicians: Vec<Politician>,
) -> Result<Vec<PoliticianResource>, sqlx::Error> {
    let mut resources = Vec::with_capacity(politicians.len());
    for politician in politicians {
        let party = party_by_id(pool, politician.party_id).await?;
        let full_name = politician_full_name(&politician);
        let assets =
            serde_json::from_value(politician.assets).unwrap_or_else(|_| Default::default());
        resources.push(PoliticianResource {
            id: politician.id.to_string(),
            slug: politician.slug,
            full_name,
            email: politician.email,
            phone: politician.phone,
            party,
            thumbnail_image_url: politician.thumbnail_image_url,
            assets,
        });
    }
    Ok(resources)
}

fn politician_full_name(politician: &Politician) -> String {
    format!(
        "{} {} {}{}",
        politician
            .preferred_name
            .as_ref()
            .unwrap_or(&politician.first_name),
        politician.middle_name.as_deref().unwrap_or(""),
        politician.last_name,
        politician
            .suffix
            .as_ref()
            .map(|suffix| format!(" {suffix}"))
            .unwrap_or_default()
    )
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

#[derive(sqlx::FromRow)]
struct CandidateVotesRow {
    candidate_id: Uuid,
    votes: Option<i32>,
}

async fn votes_for_race(
    pool: &PgPool,
    race_id: Uuid,
    total_votes: Option<i32>,
) -> Result<Vec<CandidateVotesResource>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CandidateVotesRow>(
        "SELECT candidate_id, votes FROM race_candidates WHERE race_id = $1 ORDER BY candidate_id",
    )
    .bind(race_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| CandidateVotesResource {
            candidate_id: row.candidate_id.to_string(),
            votes: row.votes,
            vote_percentage: percentage(row.votes, total_votes),
        })
        .collect())
}

async fn related_embeds_for_race(
    pool: &PgPool,
    race: &Race,
) -> Result<Vec<EmbedResource>, sqlx::Error> {
    let embeds = sqlx::query_as::<_, db::Embed>(
        r#"
        SELECT *
        FROM embed
        WHERE
            attributes->>'raceId' = $1
            OR attributes->>'politicianId' IN (
                SELECT candidate_id::text
                FROM race_candidates
                WHERE race_id = $2 AND is_running = TRUE
            )
        ORDER BY id
        "#,
    )
    .bind(race.id.to_string())
    .bind(race.id)
    .fetch_all(pool)
    .await?;

    let mut resources = Vec::with_capacity(embeds.len());
    for embed in embeds {
        let references_race = embed
            .attributes
            .get("raceId")
            .and_then(|value| value.as_str())
            == Some(race.id.to_string().as_str());
        resources.push(EmbedResource {
            id: embed.id.to_string(),
            organization_id: embed.organization_id.to_string(),
            embed_type: embed_type_name(embed.embed_type),
            race: references_race.then(|| RaceReferenceResource {
                title: race.title.clone(),
            }),
        });
    }
    Ok(resources)
}

fn embed_type_name(embed_type: db::EmbedType) -> &'static str {
    match embed_type {
        db::EmbedType::Legislation => "legislation",
        db::EmbedType::LegislationTracker => "legislation_tracker",
        db::EmbedType::Politician => "politician",
        db::EmbedType::Question => "question",
        db::EmbedType::Poll => "poll",
        db::EmbedType::Race => "race",
        db::EmbedType::CandidateGuide => "candidate_guide",
        db::EmbedType::MyBallot => "my_ballot",
        db::EmbedType::Conversation => "conversation",
    }
}

fn percentage(numerator: Option<i32>, denominator: Option<i32>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if denominator > 0 => {
            Some(((f64::from(numerator) / f64::from(denominator) * 100.0) * 10.0).round() / 10.0)
        }
        _ => None,
    }
}

fn computed_office_subtitle(office: &Office, short: bool) -> Option<String> {
    match (
        office.election_scope,
        office.political_scope,
        office.district_type,
    ) {
        (ElectionScope::National, _, _) => None,
        (ElectionScope::State, _, _) => office
            .state
            .map(|state| db::FullState::full_state(&state).to_string()),
        (ElectionScope::District, PoliticalScope::Federal, Some(DistrictType::UsCongressional)) => {
            Some(format!(
                "{} - District {}",
                office.state?,
                office.district.as_deref()?
            ))
        }
        (ElectionScope::District, PoliticalScope::State, Some(DistrictType::StateHouse)) => {
            Some(format!(
                "{} - {} {}",
                office.state?,
                if short { "HD" } else { "House District" },
                office.district.as_deref()?
            ))
        }
        (ElectionScope::District, PoliticalScope::State, Some(DistrictType::StateSenate)) => {
            Some(format!(
                "{} - {} {}",
                office.state?,
                if short { "SD" } else { "Senate District" },
                office.district.as_deref()?
            ))
        }
        (ElectionScope::District, PoliticalScope::Local, Some(DistrictType::County)) => {
            Some(format!(
                "{} County, {} - District {}",
                office.county.as_deref()?,
                office.state?,
                office.district.as_deref()?
            ))
        }
        (ElectionScope::District, PoliticalScope::Local, Some(DistrictType::City)) => {
            Some(format!(
                "{}, {} - {}",
                office.municipality.as_deref()?,
                office.state?,
                office.district.as_deref()?
            ))
        }
        (ElectionScope::District, PoliticalScope::Local, Some(DistrictType::School)) => {
            let base = format!("{} - {}", office.state?, office.school_district.as_deref()?);
            Some(match office.district.as_deref() {
                Some(district) => format!("{base} - District {district}"),
                None => base,
            })
        }
        (ElectionScope::County, _, _) => Some(format!("{} County", office.county.as_deref()?)),
        (ElectionScope::City, PoliticalScope::Local, _) => Some(format!(
            "{}, {}",
            office.municipality.as_deref()?,
            office.state?
        )),
        _ => None,
    }
}

pub(super) fn endpoint_template() -> &'static str {
    ENDPOINT_TEMPLATE
}

#[cfg(test)]
pub(super) fn test_ballot_data(election_id: Uuid) -> BallotData {
    BallotData {
        election: ElectionResource {
            id: election_id.to_string(),
            slug: "test-election".to_string(),
            title: "Test Election".to_string(),
            description: None,
            state: Some(UsState::MN),
            election_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 3)
                .expect("test date is valid"),
        },
        races: vec![RaceResource {
            id: Uuid::nil().to_string(),
            slug: "mayor".to_string(),
            title: "Mayor".to_string(),
            election_id: Some(election_id.to_string()),
            office_id: Uuid::nil().to_string(),
            race_type: db::RaceType::General,
            vote_type: db::VoteType::Plurality,
            state: Some(UsState::MN),
            election_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 3)
                .expect("test date is valid"),
            is_special_election: false,
            num_elect: Some(1),
            party: None,
            office: OfficeResource {
                id: Uuid::nil().to_string(),
                name: Some("Minneapolis Mayor".to_string()),
                title: "Mayor".to_string(),
                subtitle: Some("Minneapolis, MN".to_string()),
                subtitle_short: Some("Minneapolis, MN".to_string()),
                state: Some(UsState::MN),
                county: Some("Hennepin".to_string()),
                municipality: Some("Minneapolis".to_string()),
                district: None,
                seat: None,
                election_scope: ElectionScope::City,
                district_type: None,
                political_scope: PoliticalScope::Local,
                incumbents: Vec::new(),
            },
            candidates: Vec::new(),
            results: RaceResultsResource {
                total_votes: None,
                precinct_reporting_percentage: None,
                votes_by_candidate: Vec::new(),
                winners: Vec::new(),
            },
            related_embeds: Vec::new(),
        }],
        ballot_measures: vec![BallotMeasureResource {
            id: Uuid::nil().to_string(),
            title: "Test Measure".to_string(),
            description: Some("A test ballot measure.".to_string()),
            state: UsState::MN,
            ballot_measure_code: "Question 1".to_string(),
            yes_votes: None,
            no_votes: None,
            num_precincts_reporting: None,
            total_precincts: None,
            election_scope: Some(ElectionScope::City),
        }],
        coverage: CoverageResource::for_state(UsState::MN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reports_geographic_matching_coverage_by_state() {
        assert_eq!(
            serde_json::to_value(CoverageResource::for_state(UsState::MN)).unwrap(),
            json!({
                "races": "address_specific",
                "ballotMeasures": "address_specific",
                "warnings": []
            })
        );
        assert_eq!(
            serde_json::to_value(CoverageResource::for_state(UsState::TX)).unwrap(),
            json!({
                "races": "address_specific",
                "ballotMeasures": "statewide_only",
                "warnings": [
                    "Local ballot-measure matching is not currently available for Texas."
                ]
            })
        );
        assert_eq!(
            serde_json::to_value(CoverageResource::for_state(UsState::AL)).unwrap(),
            json!({
                "races": "statewide_only",
                "ballotMeasures": "statewide_only",
                "warnings": [
                    "District and local contest matching is not currently available for this state."
                ]
            })
        );
    }
}
