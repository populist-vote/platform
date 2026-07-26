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
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

pub use states::StateResource;

const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;
const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
const NOSNIFF: HeaderName = HeaderName::from_static("x-content-type-options");

#[derive(Debug, Clone)]
pub struct RequestId(pub String);

#[derive(Clone)]
struct RestState {
    states: Arc<[StateResource]>,
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

pub fn router(states: Vec<StateResource>) -> Router {
    let state = RestState {
        states: states.into(),
    };
    let v1 = Router::new()
        .route("/", get(index).fallback(method_not_allowed))
        .route("/health", get(health).fallback(method_not_allowed))
        .route("/states", get(states::list).fallback(method_not_allowed))
        .fallback(not_found)
        .with_state(state);

    Router::new()
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
    use axum::http::{header::CONTENT_TYPE, Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    fn test_router() -> Router {
        router(vec![
            StateResource::new("AL", "Alabama"),
            StateResource::new("AK", "Alaska"),
            StateResource::new("AS", "American Samoa"),
        ])
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
}
