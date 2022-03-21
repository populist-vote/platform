use async_graphql::{Context, Object, Result};
use auth::Claims;
use jsonwebtoken::TokenData;

use crate::{
    context::ApiContext,
    types::{Error, UserResult},
};

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Validate that a user does not already exist with this email
    async fn validate_email_available(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(email))] email: String,
    ) -> Result<bool, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        // Ensure email is not already in database
        // TODO: handle email aliases
        let existing_user = sqlx::query!(
            r#"
            SELECT id FROM populist_user WHERE email = $1
        "#,
            email
        )
        .fetch_optional(&db_pool)
        .await?;

        if let Some(_user) = existing_user {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Providers current user based on JWT found in client's access_token cookie
    async fn current_user(&self, ctx: &Context<'_>) -> Result<Option<UserResult>, Error> {
        let user = ctx.data::<Option<TokenData<Claims>>>().unwrap();

        match user {
            Some(user) => Ok(Some(UserResult::from(user))),
            None => Ok(None),
        }
    }
}
