use async_graphql::Context;
use auth::{AccessTokenClaims, AuthenticationKind};
use jsonwebtoken::TokenData;
use uuid::Uuid;

use crate::types::Error;

pub fn interactive_user_id(ctx: &Context<'_>) -> Result<Uuid, Error> {
    let authentication_kind = ctx
        .data_opt::<Option<AuthenticationKind>>()
        .copied()
        .flatten();

    if authentication_kind == Some(AuthenticationKind::ApiKey) {
        return Err(Error::InteractiveAuthenticationRequired);
    }

    ctx.data::<Option<TokenData<AccessTokenClaims>>>()
        .ok()
        .and_then(Option::as_ref)
        .map(|token| token.claims.sub)
        .ok_or(Error::Unauthorized)
}
