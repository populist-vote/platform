mod ballots;
mod elections;
mod error;
mod pagination;
mod states;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, OriginalUri, Request},
    http::{
        header::{HeaderName, HeaderValue},
        Method,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

pub use states::StateResource;

const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;
pub(crate) const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
const NOSNIFF: HeaderName = HeaderName::from_static("x-content-type-options");

#[derive(Debug, Clone)]
pub struct RequestId(pub String);

#[derive(Clone)]
struct RestState {
    states: Arc<[StateResource]>,
    ballots: Arc<dyn ballots::BallotLookup>,
    elections: elections::SharedElectionDataLookup,
}

#[derive(Serialize)]
struct ApiIndex {
    data: ApiIndexData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiIndexData {
    version: &'static str,
    health: &'static str,
    states: &'static str,
    elections: &'static str,
    election: &'static str,
    election_races: &'static str,
    election_race: &'static str,
    election_results: &'static str,
    election_ballot_measures: &'static str,
    ballot_by_address: &'static str,
}

#[derive(Serialize)]
struct Health {
    data: HealthData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthData {
    status: &'static str,
    api_version: &'static str,
}

pub fn router(states: Vec<StateResource>, pool: sqlx::PgPool) -> Router {
    router_with_lookups(
        states,
        Arc::new(ballots::DatabaseBallotLookup::new(pool.clone())),
        Arc::new(elections::DatabaseElectionDataLookup::new(pool)),
    )
}

#[cfg(test)]
fn router_with_ballot_lookup(
    states: Vec<StateResource>,
    ballots: Arc<dyn ballots::BallotLookup>,
) -> Router {
    router_with_lookups(
        states,
        ballots,
        Arc::new(elections::UnavailableElectionDataLookup),
    )
}

fn router_with_lookups(
    states: Vec<StateResource>,
    ballots: Arc<dyn ballots::BallotLookup>,
    elections: elections::SharedElectionDataLookup,
) -> Router {
    let state = RestState {
        states: states.into(),
        ballots,
        elections,
    };
    let v1 = Router::new()
        .route("/health", get(health).fallback(method_not_allowed))
        .route("/states", get(states::list).fallback(method_not_allowed))
        .route(
            "/elections",
            get(elections::list).fallback(method_not_allowed),
        )
        .route(
            "/elections/:election_id",
            get(elections::get_one).fallback(method_not_allowed),
        )
        .route(
            "/elections/:election_id/races",
            get(elections::list_races).fallback(method_not_allowed),
        )
        .route(
            "/elections/:election_id/races/:race_id",
            get(elections::get_race).fallback(method_not_allowed),
        )
        .route(
            "/elections/:election_id/results",
            get(elections::list_results).fallback(method_not_allowed),
        )
        .route(
            "/elections/:election_id/ballot-measures",
            get(elections::list_ballot_measures).fallback(method_not_allowed),
        )
        .route(
            "/elections/:election_id/ballot",
            post(ballots::lookup)
                .layer(DefaultBodyLimit::max(ballots::MAX_REQUEST_BODY_BYTES))
                .fallback(method_not_allowed),
        )
        .fallback(not_found)
        .with_state(state);

    let index_route = get(index).fallback(method_not_allowed);
    Router::new()
        .route("/api/v1", index_route.clone())
        .route("/api/v1/", index_route)
        .nest("/api/v1", v1)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
        .layer(middleware::from_fn(add_response_headers))
}

async fn index() -> Json<ApiIndex> {
    Json(ApiIndex {
        data: ApiIndexData {
            version: "v1",
            health: "/api/v1/health",
            states: "/api/v1/states",
            elections: elections::endpoint_templates()[0],
            election: elections::endpoint_templates()[1],
            election_races: elections::endpoint_templates()[2],
            election_race: elections::endpoint_templates()[3],
            election_results: elections::endpoint_templates()[4],
            election_ballot_measures: elections::endpoint_templates()[5],
            ballot_by_address: ballots::endpoint_template(),
        },
    })
}

async fn health() -> Json<Health> {
    Json(Health {
        data: HealthData {
            status: "ok",
            api_version: "v1",
        },
    })
}

async fn not_found(OriginalUri(uri): OriginalUri) -> error::ApiError {
    error::ApiError::not_found(uri.path())
}

async fn method_not_allowed(method: Method, OriginalUri(uri): OriginalUri) -> error::ApiError {
    error::ApiError::method_not_allowed(method, uri.path())
}

async fn add_response_headers(mut request: Request<Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(&REQUEST_ID)
        .filter(|value| value.as_bytes().len() <= 128 && value.to_str().is_ok())
        .cloned()
        .unwrap_or_else(|| {
            HeaderValue::from_str(&uuid::Uuid::new_v4().to_string())
                .expect("UUIDs are valid header values")
        });

    request.extensions_mut().insert(RequestId(
        request_id
            .to_str()
            .expect("request ID was validated as a header value")
            .to_owned(),
    ));

    let mut response = next.run(request).await.into_response();
    response.headers_mut().insert(REQUEST_ID, request_id);
    response
        .headers_mut()
        .insert(NOSNIFF, HeaderValue::from_static("nosniff"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::http::{
        header::{CACHE_CONTROL, CONTENT_TYPE, PRAGMA, RETRY_AFTER},
        Request, StatusCode,
    };
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use std::sync::Mutex;
    use tower::ServiceExt;

    struct TestBallotLookup;

    #[async_trait]
    impl ballots::BallotLookup for TestBallotLookup {
        async fn lookup(
            &self,
            _request: ballots::BallotLookupRequest,
        ) -> Result<ballots::BallotData, ballots::BallotLookupError> {
            Err(ballots::BallotLookupError::Internal)
        }
    }

    struct SuccessfulBallotLookup {
        request: Mutex<Option<ballots::BallotLookupRequest>>,
    }

    #[async_trait]
    impl ballots::BallotLookup for SuccessfulBallotLookup {
        async fn lookup(
            &self,
            request: ballots::BallotLookupRequest,
        ) -> Result<ballots::BallotData, ballots::BallotLookupError> {
            let data = ballots::test_ballot_data(request.election_id);
            *self.request.lock().unwrap() = Some(request);
            Ok(data)
        }
    }

    struct FailingBallotLookup(ballots::BallotLookupError);

    #[async_trait]
    impl ballots::BallotLookup for FailingBallotLookup {
        async fn lookup(
            &self,
            _request: ballots::BallotLookupRequest,
        ) -> Result<ballots::BallotData, ballots::BallotLookupError> {
            Err(self.0)
        }
    }

    struct PendingBallotLookup;

    #[async_trait]
    impl ballots::BallotLookup for PendingBallotLookup {
        async fn lookup(
            &self,
            _request: ballots::BallotLookupRequest,
        ) -> Result<ballots::BallotData, ballots::BallotLookupError> {
            std::future::pending().await
        }
    }

    #[derive(Default)]
    struct SuccessfulElectionDataLookup {
        election_filters: Mutex<Option<(elections::ElectionFilters, elections::PageRequest)>>,
        race_filters: Mutex<Option<(elections::RaceFilters, elections::PageRequest)>>,
        measure_filters: Mutex<Option<(elections::BallotMeasureFilters, elections::PageRequest)>>,
    }

    fn test_election(election_id: uuid::Uuid) -> elections::ElectionResource {
        elections::ElectionResource {
            id: election_id.to_string(),
            slug: "minnesota-2026".to_string(),
            title: "Minnesota 2026".to_string(),
            description: Some("A test election.".to_string()),
            state: Some(db::State::MN),
            municipality: None,
            election_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 3).unwrap(),
        }
    }

    fn test_office() -> elections::OfficeSummaryResource {
        elections::OfficeSummaryResource {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Mayor".to_string(),
            subtitle: Some("Minneapolis".to_string()),
            state: Some(db::State::MN),
            county: Some("Hennepin".to_string()),
            municipality: Some("Minneapolis".to_string()),
            district: None,
            seat: None,
            election_scope: db::ElectionScope::City,
            district_type: Some(db::DistrictType::City),
            political_scope: db::PoliticalScope::Local,
        }
    }

    fn test_race_summary(
        election_id: uuid::Uuid,
        race_id: uuid::Uuid,
    ) -> elections::RaceSummaryResource {
        elections::RaceSummaryResource {
            id: race_id.to_string(),
            slug: "minneapolis-mayor".to_string(),
            title: "Mayor".to_string(),
            election_id: election_id.to_string(),
            race_type: db::RaceType::General,
            vote_type: db::VoteType::Plurality,
            state: Some(db::State::MN),
            is_special_election: false,
            num_elect: Some(1),
            office: test_office(),
            party: None,
            results: elections::ResultSummaryResource {
                total_votes: Some(100),
                num_precincts_reporting: Some(10),
                total_precincts: Some(20),
                precinct_reporting_percentage: Some(50.0),
                winners: Vec::new(),
            },
            updated_at: chrono::Utc::now(),
        }
    }

    #[async_trait]
    impl elections::ElectionDataLookup for SuccessfulElectionDataLookup {
        async fn elections(
            &self,
            filters: elections::ElectionFilters,
            page: elections::PageRequest,
        ) -> Result<elections::Collection<elections::ElectionResource>, elections::ElectionDataError>
        {
            *self.election_filters.lock().unwrap() = Some((filters, page));
            Ok(elections::Collection::new(
                vec![test_election(uuid::Uuid::nil())],
                1,
                page,
            ))
        }

        async fn election(
            &self,
            election_id: uuid::Uuid,
        ) -> Result<elections::ElectionResource, elections::ElectionDataError> {
            if election_id.is_nil() {
                return Err(elections::ElectionDataError::ElectionNotFound);
            }
            Ok(test_election(election_id))
        }

        async fn races(
            &self,
            election_id: uuid::Uuid,
            filters: elections::RaceFilters,
            page: elections::PageRequest,
        ) -> Result<
            elections::Collection<elections::RaceSummaryResource>,
            elections::ElectionDataError,
        > {
            *self.race_filters.lock().unwrap() = Some((filters, page));
            Ok(elections::Collection::new(
                vec![test_race_summary(election_id, uuid::Uuid::nil())],
                1,
                page,
            ))
        }

        async fn race(
            &self,
            election_id: uuid::Uuid,
            race_id: uuid::Uuid,
            _endorser_id: Option<uuid::Uuid>,
        ) -> Result<elections::RaceDetailResource, elections::ElectionDataError> {
            if race_id.is_nil() {
                return Err(elections::ElectionDataError::RaceNotFound);
            }
            Ok(elections::RaceDetailResource {
                race: ballots::test_race_resource(election_id),
                description: Some("Race detail".to_string()),
                ballotpedia_link: None,
                early_voting_begins_date: None,
                official_website: None,
                updated_at: chrono::Utc::now(),
            })
        }

        async fn results(
            &self,
            _election_id: uuid::Uuid,
            page: elections::PageRequest,
        ) -> Result<
            elections::Collection<elections::RaceResultsFeedResource>,
            elections::ElectionDataError,
        > {
            Ok(elections::Collection::new(
                vec![elections::RaceResultsFeedResource {
                    race_id: uuid::Uuid::nil().to_string(),
                    slug: "minneapolis-mayor".to_string(),
                    title: "Mayor".to_string(),
                    office: test_office(),
                    total_votes: Some(100),
                    num_precincts_reporting: Some(10),
                    total_precincts: Some(20),
                    precinct_reporting_percentage: Some(50.0),
                    candidates: vec![elections::CandidateResultResource {
                        id: uuid::Uuid::new_v4().to_string(),
                        slug: "candidate".to_string(),
                        full_name: "Test Candidate".to_string(),
                        party: None,
                        votes: Some(60),
                        vote_percentage: Some(60.0),
                    }],
                    winners: Vec::new(),
                    updated_at: chrono::Utc::now(),
                }],
                1,
                page,
            ))
        }

        async fn ballot_measures(
            &self,
            election_id: uuid::Uuid,
            filters: elections::BallotMeasureFilters,
            page: elections::PageRequest,
        ) -> Result<
            elections::Collection<elections::BallotMeasureResource>,
            elections::ElectionDataError,
        > {
            *self.measure_filters.lock().unwrap() = Some((filters, page));
            Ok(elections::Collection::new(
                vec![elections::BallotMeasureResource {
                    id: uuid::Uuid::new_v4().to_string(),
                    slug: "question-one".to_string(),
                    title: "Question One".to_string(),
                    status: "on_the_ballot",
                    election_id: election_id.to_string(),
                    state: db::State::MN,
                    county: Some("Hennepin".to_string()),
                    municipality: Some("Minneapolis".to_string()),
                    school_district: None,
                    ballot_measure_code: "Question 1".to_string(),
                    measure_type: Some("charter_amendment".to_string()),
                    definitions: None,
                    description: Some("A measure.".to_string()),
                    official_summary: None,
                    populist_summary: None,
                    full_text_url: None,
                    yes_votes: Some(51),
                    no_votes: Some(49),
                    num_precincts_reporting: Some(10),
                    total_precincts: Some(20),
                    election_scope: Some(db::ElectionScope::City),
                    updated_at: chrono::Utc::now(),
                }],
                1,
                page,
            ))
        }
    }

    fn test_router() -> Router {
        router_with_ballot_lookup(
            vec![
                StateResource::new("AL", "Alabama"),
                StateResource::new("AK", "Alaska"),
                StateResource::new("AS", "American Samoa"),
            ],
            Arc::new(TestBallotLookup),
        )
    }

    fn test_router_with_elections(lookup: Arc<SuccessfulElectionDataLookup>) -> Router {
        router_with_lookups(Vec::new(), Arc::new(TestBallotLookup), lookup)
    }

    async fn call(method: Method, uri: &str) -> Response {
        test_router()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    async fn call_with(app: Router, method: Method, uri: &str, body: Body) -> Response {
        app.oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn json_body(response: Response) -> Value {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn lists_states_with_bounded_pagination() {
        let response = call(Method::GET, "/api/v1/states?limit=2&offset=1").await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
        assert_eq!(
            json_body(response).await,
            json!({
                "data": [
                    {"code": "AK", "name": "Alaska"},
                    {"code": "AS", "name": "American Samoa"}
                ],
                "meta": {
                    "count": 2,
                    "total": 3,
                    "limit": 2,
                    "offset": 1
                }
            })
        );
    }

    #[tokio::test]
    async fn advertises_the_ballot_endpoint() {
        for path in ["/api/v1", "/api/v1/"] {
            let response = call(Method::GET, path).await;

            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                json_body(response).await["data"]["ballotByAddress"],
                "/api/v1/elections/{electionId}/ballot"
            );
        }
    }

    #[tokio::test]
    async fn advertises_the_election_data_endpoints() {
        let response = call(Method::GET, "/api/v1").await;
        let data = json_body(response).await["data"].clone();
        assert_eq!(data["elections"], "/api/v1/elections");
        assert_eq!(data["election"], "/api/v1/elections/{electionId}");
        assert_eq!(
            data["electionRaces"],
            "/api/v1/elections/{electionId}/races"
        );
        assert_eq!(
            data["electionRace"],
            "/api/v1/elections/{electionId}/races/{raceId}"
        );
        assert_eq!(
            data["electionResults"],
            "/api/v1/elections/{electionId}/results"
        );
        assert_eq!(
            data["electionBallotMeasures"],
            "/api/v1/elections/{electionId}/ballot-measures"
        );
    }

    #[tokio::test]
    async fn serves_election_data_routes_with_stable_caching_and_filters() {
        let election_id = uuid::Uuid::new_v4();
        let race_id = uuid::Uuid::new_v4();
        let lookup = Arc::new(SuccessfulElectionDataLookup::default());
        let cases = [
            (
                format!("/api/v1/elections?state=mn&year=2026&query=primary&limit=7&offset=2"),
                "public, max-age=300, stale-while-revalidate=900",
                "Minnesota 2026",
            ),
            (
                format!("/api/v1/elections/{election_id}"),
                "public, max-age=300, stale-while-revalidate=900",
                "Minnesota 2026",
            ),
            (
                format!(
                    "/api/v1/elections/{election_id}/races?state=MN&raceType=general&politicalScope=local&electionScope=city&districtType=city&query=mayor&limit=8&offset=3"
                ),
                "public, max-age=60, stale-while-revalidate=300",
                "Mayor",
            ),
            (
                format!("/api/v1/elections/{election_id}/races/{race_id}"),
                "public, max-age=60, stale-while-revalidate=300",
                "Race detail",
            ),
            (
                format!("/api/v1/elections/{election_id}/results?limit=9&offset=4"),
                "public, max-age=5, stale-while-revalidate=25",
                "Test Candidate",
            ),
            (
                format!(
                    "/api/v1/elections/{election_id}/ballot-measures?state=mn&status=on_the_ballot&electionScope=city&county=Hennepin&municipality=Minneapolis&schoolDistrict=1&limit=6&offset=5"
                ),
                "public, max-age=60, stale-while-revalidate=300",
                "Question One",
            ),
        ];

        for (uri, cache_control, expected) in cases {
            let response = call_with(
                test_router_with_elections(lookup.clone()),
                Method::GET,
                &uri,
                Body::empty(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert_eq!(
                response.headers().get(CACHE_CONTROL).unwrap(),
                cache_control,
                "{uri}"
            );
            assert!(json_body(response).await.to_string().contains(expected));
        }

        let (filters, page) = lookup.election_filters.lock().unwrap().clone().unwrap();
        assert_eq!(filters.state, Some(db::State::MN));
        assert_eq!(filters.year, Some(2026));
        assert_eq!(filters.query.as_deref(), Some("primary"));
        assert_eq!(
            page,
            elections::PageRequest {
                limit: 7,
                offset: 2
            }
        );

        let (filters, page) = lookup.race_filters.lock().unwrap().clone().unwrap();
        assert_eq!(filters.state, Some(db::State::MN));
        assert_eq!(filters.race_type, Some(db::RaceType::General));
        assert_eq!(filters.political_scope, Some(db::PoliticalScope::Local));
        assert_eq!(filters.election_scope, Some(db::ElectionScope::City));
        assert_eq!(filters.district_type, Some(db::DistrictType::City));
        assert_eq!(filters.query.as_deref(), Some("mayor"));
        assert_eq!(
            page,
            elections::PageRequest {
                limit: 8,
                offset: 3
            }
        );

        let (filters, page) = lookup.measure_filters.lock().unwrap().clone().unwrap();
        assert_eq!(filters.state, Some(db::State::MN));
        assert_eq!(filters.status, Some("on_the_ballot"));
        assert_eq!(filters.election_scope, Some(db::ElectionScope::City));
        assert_eq!(filters.county.as_deref(), Some("Hennepin"));
        assert_eq!(filters.municipality.as_deref(), Some("Minneapolis"));
        assert_eq!(filters.school_district.as_deref(), Some("1"));
        assert_eq!(
            page,
            elections::PageRequest {
                limit: 6,
                offset: 5
            }
        );
    }

    #[tokio::test]
    async fn rejects_invalid_election_data_parameters_without_calling_the_lookup() {
        let election_id = uuid::Uuid::new_v4();
        for uri in [
            "/api/v1/elections?year=1700",
            "/api/v1/elections?state=not-a-state",
            "/api/v1/elections?unexpected=true",
            "/api/v1/elections/not-a-uuid",
            &format!("/api/v1/elections/{election_id}/races?raceType=runoff"),
            &format!("/api/v1/elections/{election_id}/results?limit=101"),
            &format!("/api/v1/elections/{election_id}/ballot-measures?status=not-a-status"),
        ] {
            let response = call_with(
                test_router_with_elections(Arc::new(SuccessfulElectionDataLookup::default())),
                Method::GET,
                uri,
                Body::empty(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
            assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        }
    }

    #[tokio::test]
    async fn maps_missing_elections_and_nested_races_to_stable_not_found_problems() {
        let lookup = Arc::new(SuccessfulElectionDataLookup::default());
        for (uri, resource) in [
            (
                format!("/api/v1/elections/{}", uuid::Uuid::nil()),
                "election",
            ),
            (
                format!(
                    "/api/v1/elections/{}/races/{}",
                    uuid::Uuid::new_v4(),
                    uuid::Uuid::nil()
                ),
                "race",
            ),
        ] {
            let response = call_with(
                test_router_with_elections(lookup.clone()),
                Method::GET,
                &uri,
                Body::empty(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
            let body = json_body(response).await;
            assert_eq!(body["code"], "resource_not_found");
            assert!(body["detail"].as_str().unwrap().contains(resource));
        }
    }

    #[tokio::test]
    async fn rejects_invalid_pagination_as_problem_json() {
        let response = call(Method::GET, "/api/v1/states?limit=101").await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/problem+json"
        );
        assert_eq!(
            json_body(response).await,
            json!({
                "type": "about:blank",
                "title": "Bad Request",
                "status": 400,
                "code": "invalid_parameter",
                "detail": "Query parameter `limit` must be between 1 and 100.",
                "instance": "/api/v1/states"
            })
        );
    }

    #[tokio::test]
    async fn returns_problem_json_for_unknown_routes_and_methods() {
        let not_found = call(Method::GET, "/api/v1/unknown").await;
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            not_found.headers().get(CONTENT_TYPE).unwrap(),
            "application/problem+json"
        );

        let method_not_allowed = call(Method::POST, "/api/v1/states").await;
        assert_eq!(method_not_allowed.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(
            method_not_allowed.headers().get(CONTENT_TYPE).unwrap(),
            "application/problem+json"
        );
    }

    #[tokio::test]
    async fn echoes_valid_request_ids_and_sets_security_headers() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header(&REQUEST_ID, "caller-request-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.headers().get(&REQUEST_ID).unwrap(),
            "caller-request-id"
        );
        assert_eq!(response.headers().get(&NOSNIFF).unwrap(), "nosniff");
    }

    #[tokio::test]
    async fn generates_a_request_id_when_one_is_not_supplied() {
        let response = call(Method::GET, "/api/v1/health").await;
        let request_id = response.headers().get(&REQUEST_ID).unwrap();

        assert!(uuid::Uuid::parse_str(request_id.to_str().unwrap()).is_ok());
    }

    #[tokio::test]
    async fn replaces_non_text_request_ids() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header(&REQUEST_ID, HeaderValue::from_bytes(b"\xFF").unwrap())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let request_id = response.headers().get(&REQUEST_ID).unwrap();

        assert!(uuid::Uuid::parse_str(request_id.to_str().unwrap()).is_ok());
    }

    #[tokio::test]
    async fn returns_a_ballot_without_echoing_the_address() {
        let election_id = uuid::Uuid::new_v4();
        let endorser_id = uuid::Uuid::new_v4();
        let lookup = Arc::new(SuccessfulBallotLookup {
            request: Mutex::new(None),
        });
        let app = router_with_ballot_lookup(Vec::new(), lookup.clone());
        let response = call_with(
            app,
            Method::POST,
            &format!("/api/v1/elections/{election_id}/ballot?endorserId={endorser_id}"),
            Body::from(
                json!({
                    "address": {
                        "line1": " 123 Main St ",
                        "line2": " Apt 4 ",
                        "city": " Minneapolis ",
                        "state": "mn",
                        "postalCode": "55401",
                        "country": "usa"
                    }
                })
                .to_string(),
            ),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        assert_eq!(response.headers().get(PRAGMA).unwrap(), "no-cache");
        let body = json_body(response).await;
        assert_eq!(body["data"]["election"]["id"], election_id.to_string());
        assert_eq!(body["data"]["races"][0]["title"], "Mayor");
        assert_eq!(
            body["data"]["races"][0]["office"]["politicalScope"],
            "local"
        );
        assert_eq!(
            body["data"]["races"][0]["results"]["votesByCandidate"],
            json!([])
        );
        assert_eq!(
            body["data"]["ballotMeasures"][0]["ballotMeasureCode"],
            "Question 1"
        );
        assert_eq!(
            body["data"]["coverage"],
            json!({
                "races": "address_specific",
                "ballotMeasures": "address_specific",
                "warnings": []
            })
        );
        assert!(!body.to_string().contains("123 Main St"));
        assert!(!body.to_string().contains("Apt 4"));

        let captured = lookup.request.lock().unwrap();
        let request = captured.as_ref().unwrap();
        assert_eq!(request.election_id, election_id);
        assert_eq!(request.endorser_id, Some(endorser_id));
        assert_eq!(request.address.line_1, "123 Main St");
        assert_eq!(request.address.line_2, None);
        assert_eq!(request.address.city, "Minneapolis");
        assert_eq!(request.address.state, db::State::MN);
        assert_eq!(request.address.country, "US");
    }

    #[tokio::test]
    async fn rejects_malformed_and_unknown_json_fields() {
        let election_id = uuid::Uuid::new_v4();
        for body in [
            Body::from("{"),
            Body::from(
                json!({
                    "address": {
                        "line1": "123 Main St",
                        "city": "Minneapolis",
                        "state": "MN",
                        "postalCode": "55401",
                        "coordinates": {"latitude": 1, "longitude": 2}
                    }
                })
                .to_string(),
            ),
        ] {
            let response = call_with(
                test_router(),
                Method::POST,
                &format!("/api/v1/elections/{election_id}/ballot"),
                body,
            )
            .await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            assert_eq!(
                response.headers().get(CONTENT_TYPE).unwrap(),
                "application/problem+json"
            );
            assert_eq!(json_body(response).await["code"], "invalid_body");
        }
    }

    #[tokio::test]
    async fn rejects_unknown_query_parameters() {
        let election_id = uuid::Uuid::new_v4();
        let response = call_with(
            test_router(),
            Method::POST,
            &format!("/api/v1/elections/{election_id}/ballot?unexpected=true"),
            Body::from(
                json!({
                    "address": {
                        "line1": "123 Main St",
                        "city": "Minneapolis",
                        "state": "MN",
                        "postalCode": "55401"
                    }
                })
                .to_string(),
            ),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(json_body(response).await["code"], "malformed_query");
    }

    #[tokio::test]
    async fn rejects_missing_json_content_type_and_oversized_bodies() {
        let election_id = uuid::Uuid::new_v4();
        let uri = format!("/api/v1/elections/{election_id}/ballot");
        let missing_content_type = test_router()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(&uri)
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            missing_content_type.status(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        );
        assert_eq!(
            json_body(missing_content_type).await["code"],
            "unsupported_media_type"
        );

        let oversized = call_with(
            test_router(),
            Method::POST,
            &uri,
            Body::from("x".repeat(ballots::MAX_REQUEST_BODY_BYTES + 1)),
        )
        .await;
        assert_eq!(oversized.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json_body(oversized).await["code"], "payload_too_large");
    }

    #[tokio::test]
    async fn rejects_invalid_address_fields_as_unprocessable() {
        let election_id = uuid::Uuid::new_v4();
        for (field, value) in [
            ("state", "Minnesota"),
            ("postalCode", "not-a-zip"),
            ("country", "CA"),
        ] {
            let mut address = json!({
                "line1": "123 Main St",
                "city": "Minneapolis",
                "state": "MN",
                "postalCode": "55401"
            });
            address[field] = Value::String(value.to_string());
            let response = call_with(
                test_router(),
                Method::POST,
                &format!("/api/v1/elections/{election_id}/ballot"),
                Body::from(json!({"address": address}).to_string()),
            )
            .await;
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(json_body(response).await["code"], "invalid_address");
        }
    }

    #[tokio::test]
    async fn rejects_invalid_path_and_query_ids() {
        let body = Body::from(
            json!({
                "address": {
                    "line1": "123 Main St",
                    "city": "Minneapolis",
                    "state": "MN",
                    "postalCode": "55401"
                }
            })
            .to_string(),
        );
        let response = call_with(
            test_router(),
            Method::POST,
            "/api/v1/elections/not-a-uuid/ballot",
            body,
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(json_body(response).await["code"], "invalid_path_parameter");

        let response = call_with(
            test_router(),
            Method::POST,
            &format!(
                "/api/v1/elections/{}/ballot?endorserId=nope",
                uuid::Uuid::new_v4()
            ),
            Body::from(
                json!({
                    "address": {
                        "line1": "123 Main St",
                        "city": "Minneapolis",
                        "state": "MN",
                        "postalCode": "55401"
                    }
                })
                .to_string(),
            ),
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(json_body(response).await["code"], "invalid_parameter");
    }

    #[tokio::test]
    async fn maps_lookup_failures_to_stable_problem_responses() {
        let election_id = uuid::Uuid::new_v4();
        let request_body = || {
            Body::from(
                json!({
                    "address": {
                        "line1": "123 Main St",
                        "city": "Minneapolis",
                        "state": "MN",
                        "postalCode": "55401"
                    }
                })
                .to_string(),
            )
        };
        let cases = [
            (
                ballots::BallotLookupError::ElectionNotFound,
                StatusCode::NOT_FOUND,
                "resource_not_found",
            ),
            (
                ballots::BallotLookupError::InvalidAddress,
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_address",
            ),
            (
                ballots::BallotLookupError::AddressServiceUnavailable,
                StatusCode::SERVICE_UNAVAILABLE,
                "address_service_unavailable",
            ),
            (
                ballots::BallotLookupError::RateLimited,
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
            ),
            (
                ballots::BallotLookupError::Internal,
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
            ),
        ];

        for (error, status, code) in cases {
            let app = router_with_ballot_lookup(Vec::new(), Arc::new(FailingBallotLookup(error)));
            let response = call_with(
                app,
                Method::POST,
                &format!("/api/v1/elections/{election_id}/ballot"),
                request_body(),
            )
            .await;
            assert_eq!(response.status(), status);
            assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
            if matches!(
                status,
                StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE
            ) {
                assert_eq!(response.headers().get(RETRY_AFTER).unwrap(), "5");
            }
            assert_eq!(json_body(response).await["code"], code);
        }
    }

    #[tokio::test]
    async fn times_out_stalled_ballot_lookups() {
        let election_id = uuid::Uuid::new_v4();
        let app = router_with_ballot_lookup(Vec::new(), Arc::new(PendingBallotLookup));
        let response = call_with(
            app,
            Method::POST,
            &format!("/api/v1/elections/{election_id}/ballot"),
            Body::from(
                json!({
                    "address": {
                        "line1": "123 Main St",
                        "city": "Minneapolis",
                        "state": "MN",
                        "postalCode": "55401"
                    }
                })
                .to_string(),
            ),
        )
        .await;

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        assert_eq!(json_body(response).await["code"], "request_timeout");
    }

    #[tokio::test]
    async fn rejects_non_post_ballot_requests() {
        let response = call(
            Method::GET,
            &format!("/api/v1/elections/{}/ballot", uuid::Uuid::new_v4()),
        )
        .await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(json_body(response).await["code"], "method_not_allowed");
    }
}
