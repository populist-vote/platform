use axum::{
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

const PROBLEM_JSON: HeaderValue = HeaderValue::from_static("application/problem+json");

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    detail: String,
    instance: String,
}

#[derive(Serialize)]
struct ProblemDetails {
    #[serde(rename = "type")]
    type_uri: &'static str,
    title: &'static str,
    status: u16,
    code: &'static str,
    detail: String,
    instance: String,
}

impl ApiError {
    pub fn invalid_parameter(
        parameter: &'static str,
        requirement: impl Into<String>,
        instance: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_parameter",
            detail: format!("Query parameter `{parameter}` {}.", requirement.into()),
            instance: instance.into(),
        }
    }

    pub fn malformed_query(detail: impl Into<String>, instance: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "malformed_query",
            detail: detail.into(),
            instance: instance.into(),
        }
    }

    pub fn method_not_allowed(method: impl std::fmt::Display, instance: impl Into<String>) -> Self {
        let instance = instance.into();
        Self {
            status: StatusCode::METHOD_NOT_ALLOWED,
            code: "method_not_allowed",
            detail: format!("Method {method} is not supported for `{instance}`."),
            instance,
        }
    }

    pub fn not_found(instance: impl Into<String>) -> Self {
        let instance = instance.into();
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            detail: format!("No REST endpoint exists at `{instance}`."),
            instance,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let problem = ProblemDetails {
            type_uri: "about:blank",
            title: self.status.canonical_reason().unwrap_or("HTTP error"),
            status: self.status.as_u16(),
            code: self.code,
            detail: self.detail,
            instance: self.instance,
        };
        let mut response = (self.status, Json(problem)).into_response();
        response.headers_mut().insert(CONTENT_TYPE, PROBLEM_JSON);
        response
    }
}
