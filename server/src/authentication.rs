use auth::{authenticate_api_key, jwt, AuthenticationKind, RequestAuthentication, API_KEY_PREFIX};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

pub async fn resolve_bearer_authentication(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Option<RequestAuthentication> {
    let header = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = token.trim();

    if token.is_empty() {
        return None;
    }

    if token.starts_with(API_KEY_PREFIX) {
        return match authenticate_api_key(pool, token).await {
            Ok(Some(token_data)) => Some(RequestAuthentication {
                claims: token_data.claims,
                kind: AuthenticationKind::ApiKey,
            }),
            Ok(None) => None,
            Err(error) => {
                tracing::error!(error = %error, "API key authentication failed");
                None
            }
        };
    }

    jwt::validate_access_token(token)
        .ok()
        .map(|token_data| RequestAuthentication {
            claims: token_data.claims,
            kind: AuthenticationKind::Jwt,
        })
}

pub async fn authentication_middleware(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(authentication) = resolve_bearer_authentication(&pool, request.headers()).await {
        request.extensions_mut().insert(authentication);
    }

    next.run(request).await
}
