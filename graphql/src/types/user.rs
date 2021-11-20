use async_graphql::{SimpleObject, ID};
use db::User;

#[derive(SimpleObject)]
pub struct UserResult {
    username: String,
    id: ID,
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
    pub access_token: String,
}
