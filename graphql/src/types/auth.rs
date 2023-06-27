use crate::{context::ApiContext, is_admin, Error};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use auth::AccessTokenClaims;
use db::{Role, User, UserWithProfile};
use jsonwebtoken::TokenData;

use super::UserResult;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
#[graphql(visible = "is_admin")]
pub struct AuthTokenResult {
    id: ID,
    username: String,
    email: String,
    role: Role,
    organization_id: Option<ID>,
}

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
pub struct CreateUserResult {
    id: ID,
}

impl From<User> for CreateUserResult {
    fn from(u: User) -> Self {
        Self { id: ID::from(u.id) }
    }
}

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
pub struct LoginResult {
    pub user_id: ID,
}

#[ComplexObject]
impl AuthTokenResult {
    async fn user_profile(&self, ctx: &Context<'_>) -> Result<UserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            UserWithProfile,
            r#"
            SELECT u.id, u.username, u.email, first_name, last_name, profile_picture_url FROM user_profile up
            JOIN populist_user u ON up.user_id = u.id WHERE u.id = $1
        "#,
            uuid::Uuid::parse_str(&self.id).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        match record {
            Some(user) => Ok(user.into()),
            None => Err(Error::UserNotFound.into()),
        }
    }
}

impl From<&TokenData<AccessTokenClaims>> for AuthTokenResult {
    fn from(user: &TokenData<AccessTokenClaims>) -> Self {
        Self {
            id: ID::from(user.claims.sub),
            username: user.claims.username.clone(),
            email: user.claims.email.clone(),
            role: user.claims.role,
            organization_id: user.claims.organization_id.map(ID::from),
        }
    }
}
