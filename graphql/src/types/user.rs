use super::AddressResult;
use crate::{context::ApiContext, is_admin, mutation::StaffOnly};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{models::enums::State, Address};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct UserResult {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[ComplexObject]
impl UserResult {
    #[graphql(guard = "StaffOnly", visible = "is_admin")] // Change this to be a guard if user = requesting user.
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
