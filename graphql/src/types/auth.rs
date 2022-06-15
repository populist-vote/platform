use async_graphql::{SimpleObject, ID};
use auth::Claims;
use db::{Role, User};
use jsonwebtoken::TokenData;

#[derive(SimpleObject, Debug, Clone)]
pub struct AuthTokenResult {
    id: ID,
    username: String,
    email: String,
    role: Role,
}

#[derive(SimpleObject)]
pub struct CreateUserResult {
    id: ID,
}

impl From<User> for CreateUserResult {
    fn from(u: User) -> Self {
        Self { id: ID::from(u.id) }
    }
}

#[derive(SimpleObject)]
pub struct LoginResult {
    pub user_id: ID,
}

impl From<&TokenData<Claims>> for AuthTokenResult {
    fn from(user: &TokenData<Claims>) -> Self {
        Self {
            id: ID::from(user.claims.sub),
            username: user.claims.username.clone(),
            email: user.claims.email.clone(),
            role: user.claims.role,
        }
    }
}
