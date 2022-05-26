use async_graphql::{Context, Object, Result, SimpleObject};
use auth::Claims;
use jsonwebtoken::TokenData;
use zxcvbn::zxcvbn;

use crate::{
    context::ApiContext,
    types::{AuthTokenResult, Error},
};

#[derive(Default)]
pub struct AuthQuery;

#[derive(Default, Debug, SimpleObject)]
pub struct PasswordEntropyResult {
    pub valid: bool,
    pub score: u8,
    pub message: Option<String>,
}

#[Object]
impl AuthQuery {
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

    async fn validate_password_entropy(
        &self,
        password: String,
    ) -> Result<PasswordEntropyResult, Error> {
        let estimate = zxcvbn(&password, &[]).unwrap();
        if estimate.score() < 3 {
            return Ok(PasswordEntropyResult {
                valid: false,
                score: estimate.score(),
                message: Some("Your password is not strong enough".to_string()),
            });
        }
        Ok(PasswordEntropyResult {
            valid: true,
            score: estimate.score(),
            message: None,
        })
    }

    /// Provides current user based on JWT found in client's access_token cookie
    async fn current_user(&self, ctx: &Context<'_>) -> Result<Option<AuthTokenResult>, Error> {
        let user = ctx.data::<Option<TokenData<Claims>>>().unwrap();

        match user {
            Some(user) => Ok(Some(AuthTokenResult::from(user))),
            None => Ok(None),
        }
    }
}
