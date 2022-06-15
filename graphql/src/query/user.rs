use async_graphql::{Context, Object, Result, ID};

use crate::{
    context::ApiContext,
    types::{Error, UserResult},
};

use db::UserWithProfile;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Publicly accessible user information
    async fn user_profile(&self, ctx: &Context<'_>, user_id: ID) -> Result<UserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            UserWithProfile,
            r#"
            SELECT u.id, u.username, u.email, first_name, last_name FROM user_profile up
            JOIN populist_user u ON up.user_id = u.id WHERE u.id = $1
        "#,
            uuid::Uuid::parse_str(user_id.as_str()).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        match record {
            Some(user) => Ok(user.into()),
            None => Err(Error::UserNotFound.into()),
        }
    }
}
