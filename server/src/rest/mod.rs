mod ballots;
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
    router_with_ballot_lookup(states, Arc::new(ballots::DatabaseBallotLookup::new(pool)))
}

fn router_with_ballot_lookup(
    states: Vec<StateResource>,
    ballots: Arc<dyn ballots::BallotLookup>,
) -> Router {
    let state = RestState {
        states: states.into(),
        ballots,
    };
    let v1 = Router::new()
        .route("/health", get(health).fallback(method_not_allowed))
        .route("/states", get(states::list).fallback(method_not_allowed))
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
