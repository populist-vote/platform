use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use auth::Claims;
use db::{models::enums::State, Address, Role, User};
use jsonwebtoken::TokenData;

use crate::context::ApiContext;

use super::AddressResult;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct UserResult {
    id: ID,
    username: String,
    email: String,
    role: Role,
}

#[ComplexObject]
impl UserResult {
    async fn address(&self, ctx: &Context<'_>) -> Result<Option<AddressResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(Address,
            r#"
            SELECT a.id, a.line_1, a.line_2, a.city, a.state AS "state:State", a.postal_code, a.country, a.congressional_district, a.state_senate_district, a.state_house_district FROM address AS a
            JOIN user_profile up ON user_id = $1
            JOIN address ON up.address_id = a.id
        "#,
            uuid::Uuid::parse_str(&self.id)?,
        )
        .fetch_optional(&db_pool)
        .await.unwrap();

        match record {
            Some(address) => Ok(Some(address.into())),
            None => Ok(None),
        }
    }
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

impl From<&TokenData<Claims>> for UserResult {
    fn from(user: &TokenData<Claims>) -> Self {
        Self {
            id: ID::from(user.claims.sub),
            username: user.claims.username.clone(),
            email: user.claims.email.clone(),
            role: user.claims.role,
        }
    }
}
