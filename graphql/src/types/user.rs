use super::AddressResult;
use crate::{context::ApiContext, guard::UserGuard, is_admin};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use db::{models::enums::State, Address, UserWithProfile};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct UserResult {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateUserProfileInput {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[ComplexObject]
impl UserResult {
    #[graphql(guard = "UserGuard::new(&self.id)", visible = "is_admin")]
    async fn address(&self, ctx: &Context<'_>) -> Result<Option<AddressResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(Address,
            r#"
            SELECT a.id, a.line_1, a.line_2, a.city, a.county, a.state AS "state:State", a.postal_code, a.country, a.congressional_district, a.state_senate_district, a.state_house_district FROM address AS a
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

impl From<UserWithProfile> for UserResult {
    fn from(user: UserWithProfile) -> Self {
        Self {
            id: user.id.into(),
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
        }
    }
}
