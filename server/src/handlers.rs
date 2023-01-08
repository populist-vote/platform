use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use auth::jwt;
use axum::{
    extract::Extension,
    http::HeaderMap,
    response::{self, IntoResponse},
};
use db::Role;
use graphql::PopulistSchema;
use tower_cookies::Cookies;

pub async fn internal_graphql_handler(
    headers: HeaderMap,
    cookies: Cookies,
    schema: Extension<PopulistSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let bearer_token = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.split_whitespace().nth(1));

    let bearer_token_data = if let Some(token) = bearer_token {
        let token_data = jwt::validate_token(token);
        if let Ok(token_data) = token_data {
            Some(token_data)
        } else {
            None
        }
    } else {
        None
    };

    let cookie = cookies.get("access_token");
    let cookie_token_data = if let Some(cookie) = cookie {
        let token_data = jwt::validate_token(cookie.value());
        if let Ok(token_data) = token_data {
            Some(token_data)
        } else {
            None
        }
    } else {
        None
    };

    // Use the bearer token if it's present, otherwise use the cookie
    let token_data = bearer_token_data.or(cookie_token_data);

    schema
        .execute(req.into_inner().data(token_data))
        .await
        .into()
}

pub async fn external_graphql_handler(
    headers: HeaderMap,
    schema: Extension<PopulistSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let err = GraphQLResponse::from(async_graphql::Response::from_errors(vec![
        async_graphql::ServerError::new("Unauthorized", None),
    ]));

    let bearer_token = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.split_whitespace().nth(1));

    let bearer_token_data = if let Some(token) = bearer_token {
        let token_data = jwt::validate_token(token);
        if let Ok(token_data) = token_data {
            Some(token_data)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(token_data) = bearer_token_data {
        if token_data.claims.role == Role::SUPERUSER || token_data.claims.role == Role::PREMIUM {
            schema
                .execute(req.into_inner().data(token_data))
                .await
                .into()
        } else {
            err
        }
    } else {
        err
    }
}

pub async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}
