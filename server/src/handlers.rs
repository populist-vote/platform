use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_poem::{GraphQLRequest, GraphQLResponse};
use auth::jwt;
use config::Environment;
use db::Role;
use graphql::PopulistSchema;
use poem::{
    handler,
    http::{HeaderMap, StatusCode},
    web::{cookie::CookieJar, Data, Html},
    Error, IntoResponse,
};
use std::str::FromStr;

use crate::{determine_request_type, RequestType};

#[handler]
pub async fn graphql_handler(
    schema: Data<&PopulistSchema>,
    req: GraphQLRequest,
    headers: &HeaderMap,
    cookie_jar: &CookieJar,
) -> poem::Result<GraphQLResponse, Error> {
    let environment = Environment::from_str(&std::env::var("ENVIRONMENT").unwrap()).unwrap();

    // Check environment to determine which origins do not require a bearer token
    let request_type = determine_request_type(environment, headers);

    tracing::info!("Request type: {:?}", request_type);

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

    let cookie = cookie_jar.get("access_token");
    let cookie_token_data = if let Some(cookie) = cookie {
        let token_data = jwt::validate_token(cookie.value_str());
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

    // Internal requests can be processed without a valid bearer token or cookie
    // External requests require a valid bearer token with a premium or superuser role
    match request_type {
        RequestType::Internal => Ok(schema.execute(req.0.data(token_data)).await.into()),
        RequestType::External => {
            if let Some(token_data) = token_data {
                if token_data.claims.role == Role::PREMIUM
                    || token_data.claims.role == Role::SUPERUSER
                {
                    Ok(schema.execute(req.0.data(token_data)).await.into())
                } else {
                    Err(poem::Error::from_status(StatusCode::UNAUTHORIZED))
                }
            } else {
                Err(poem::Error::from_status(StatusCode::UNAUTHORIZED))
            }
        }
    }
}

#[handler]
pub fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}
