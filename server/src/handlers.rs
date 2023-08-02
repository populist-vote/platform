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
use tower_cookies::{Cookie, Cookies};

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
        let token_data = jwt::validate_access_token(token);
        if let Ok(token_data) = token_data {
            Some(token_data)
        } else {
            None
        }
    } else {
        None
    };

    let refresh_token_check = if let Some(refresh_cookie) = cookies.get("refresh_token") {
        match jwt::validate_refresh_token(refresh_cookie.value()) {
            Ok(token_data) => {
                let db_pool = db::pool().await;
                let user = db::User::find_by_id(&db_pool.connection, token_data.claims.sub)
                    .await
                    .unwrap();
                if user.clone().refresh_token.unwrap() != refresh_cookie.value() {
                    cookies.to_owned().remove(Cookie::named("access_token"));
                    cookies.to_owned().remove(Cookie::named("refresh_token"));
                    None
                } else {
                    let access_token = jwt::create_access_token_for_user(user).unwrap();
                    // Set the new access token in the cookie
                    let cookie = tower_cookies::Cookie::new("access_token", access_token.clone());
                    cookies.add(cookie);
                    Some(jwt::validate_access_token(&access_token).unwrap())
                }
            }
            Err(_) => {
                cookies.to_owned().remove(Cookie::named("access_token"));
                cookies.to_owned().remove(Cookie::named("refresh_token"));
                None
            }
        }
    } else {
        None
    };

    let cookie_token_data = match cookies.get("access_token") {
        Some(access_cookie) => match jwt::validate_access_token(access_cookie.value()) {
            Ok(token_data) => Some(token_data),
            Err(_) => refresh_token_check,
        },
        None => refresh_token_check,
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
        let token_data = jwt::validate_access_token(token);
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
