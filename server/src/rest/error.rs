use axum::{
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE, RETRY_AFTER},
        HeaderValue, StatusCode,
    },
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

    pub fn invalid_path_parameter(
        parameter: &'static str,
        requirement: impl Into<String>,
        instance: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_path_parameter",
            detail: format!("Path parameter `{parameter}` {}.", requirement.into()),
            instance: instance.into(),
        }
    }

    pub fn invalid_body(detail: impl Into<String>, instance: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_body",
            detail: detail.into(),
            instance: instance.into(),
        }
    }

    pub fn unsupported_media_type(instance: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            code: "unsupported_media_type",
            detail: "The request must use `Content-Type: application/json`.".to_string(),
            instance: instance.into(),
        }
    }

    pub fn payload_too_large(instance: impl Into<String>) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            code: "payload_too_large",
            detail: "The request body exceeds the maximum allowed size.".to_string(),
            instance: instance.into(),
        }
    }

    pub fn rate_limited(instance: impl Into<String>) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
            detail: "The ballot lookup service is at capacity. Retry the request later."
                .to_string(),
            instance: instance.into(),
        }
    }

    pub fn gateway_timeout(instance: impl Into<String>) -> Self {
        Self {
            status: StatusCode::GATEWAY_TIMEOUT,
            code: "request_timeout",
            detail: "The ballot lookup did not complete before the request deadline.".to_string(),
            instance: instance.into(),
        }
    }

    pub fn unprocessable(
        code: &'static str,
        detail: impl Into<String>,
        instance: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code,
            detail: detail.into(),
            instance: instance.into(),
        }
    }

    pub fn resource_not_found(
        resource: &'static str,
        id: impl std::fmt::Display,
        instance: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "resource_not_found",
            detail: format!("No {resource} exists with ID `{id}`."),
            instance: instance.into(),
        }
    }

    pub fn service_unavailable(
        code: &'static str,
        detail: impl Into<String>,
        instance: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code,
            detail: detail.into(),
            instance: instance.into(),
        }
    }

    pub fn internal(instance: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            detail: "The server could not complete the request.".to_string(),
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
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
        if matches!(
            self.status,
            StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE
        ) {
            response
                .headers_mut()
                .insert(RETRY_AFTER, HeaderValue::from_static("5"));
        }
        response
    }
}
