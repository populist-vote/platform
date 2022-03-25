use crate::{
    context::ApiContext,
    mutation::StaffOnly,
    types::{CreateUserResult, Error, LoginResult},
};
use async_graphql::*;
use auth::{create_access_token_for_user, create_random_token, create_temporary_username, Claims};
use db::{Address, CreateUserInput, CreateUserWithProfileInput, User};
use jsonwebtoken::TokenData;
use mailers::EmailClient;
use poem::http::header::SET_COOKIE;
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};

#[derive(InputObject)]
pub struct LoginInput {
    email_or_username: String,
    password: String,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct BeginUserRegistrationInput {
    pub email: String,
    pub password: String,
    pub address: Address,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct ResetPasswordInput {
    new_password: String,
    reset_token: String,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct UpdatePasswordInput {
    old_password: String,
    new_password: String,
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    #[graphql(guard = "StaffOnly")]
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: CreateUserInput,
    ) -> Result<CreateUserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = User::create(&db_pool, &input).await?;

        Ok(CreateUserResult::from(new_record))
    }

    async fn begin_user_registration(
        &self,
        ctx: &Context<'_>,
        input: BeginUserRegistrationInput,
    ) -> Result<LoginResult, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        // Can call validate_email query prior to this mutation for UX purposes
        // Ensure email is not already in database
        // TODO: handle email aliases
        let existing_user = sqlx::query!(
            r#"
            SELECT id FROM populist_user WHERE email = $1
        "#,
            input.email
        )
        .fetch_optional(&db_pool)
        .await?;

        if let Some(_user) = existing_user {
            return Err(Error::UserExistsError);
        };

        // Create a temporary user account (unconfirmed) in the database
        let temp_username = create_temporary_username(input.email.clone());

        // Create confirmation token so user can securely confirm their email is legitimate
        let confirmation_token = create_random_token().unwrap();

        let new_user_input = CreateUserWithProfileInput {
            email: input.email.clone(),
            username: temp_username,
            password: input.password,
            address: input.address,
            first_name: input.first_name,
            last_name: input.last_name,
            confirmation_token: confirmation_token.clone(),
        };

        let new_record = User::create_with_profile(&db_pool, &new_user_input).await;

        match new_record {
            Ok(new_user) => {
                // Create Access Token to log user in
                let access_token = create_access_token_for_user(new_user.clone())?;

                let account_confirmation_url = format!(
                    "https://www.populist.us/auth/confirm?token={}",
                    confirmation_token
                );

                EmailClient::default()
                    .send_welcome_email(new_user.email, account_confirmation_url)
                    .await;

                ctx.insert_http_header(
                    SET_COOKIE,
                    format!(
                        "access_token={}; HttpOnly; SameSite=None; Secure",
                        access_token
                    ),
                );

                Ok(LoginResult {
                    user_id: new_user.id.into(),
                })
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn confirm_user_email(
        &self,
        ctx: &Context<'_>,
        confirmation_token: String,
    ) -> Result<bool, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        // Look up user in db by ID, set confirmed_at time, nullify confirmation_token
        let confirmed_user_result = sqlx::query!(
            r#"
            UPDATE populist_user 
            SET confirmed_at = now() AT TIME ZONE 'utc',
                confirmation_token = NULL
            WHERE confirmation_token = $1 
            AND confirmed_at IS NULL
            RETURNING id
        "#,
            confirmation_token
        )
        .fetch_one(&db_pool)
        .await;

        if let Ok(_confirmed) = confirmed_user_result {
            Ok(true)
        } else {
            Err(Error::ConfirmationError)
        }
    }

    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResult, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let email_or_username = input.email_or_username.to_lowercase();
        let user_lookup = User::find_by_email_or_username(&db_pool, email_or_username).await;

        if let Ok(user) = user_lookup {
            let password_is_valid = bcrypt::verify(input.password, &user.password);

            if password_is_valid {
                let access_token = create_access_token_for_user(user.clone())?;

                ctx.insert_http_header(
                    SET_COOKIE,
                    format!(
                        "access_token={}; HttpOnly; SameSite=None; Secure",
                        access_token
                    ),
                );

                Ok(LoginResult {
                    user_id: user.id.into(),
                })
            } else {
                Err(Error::PasswordError)
            }
        } else {
            Err(Error::EmailOrUsernameNotFound)
        }
    }

    async fn request_password_reset(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> Result<bool, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        if let Ok(true) = User::validate_email_exists(&db_pool, email.clone()).await {
            let reset_token = create_random_token().unwrap();
            let reset_token_expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

            let _setup_reset_request = sqlx::query!(
                r#"
                UPDATE populist_user 
                SET reset_token = $1,
                    reset_token_expires_at = $2
                WHERE email = LOWER($3)
            "#,
                reset_token,
                reset_token_expires_at,
                email
            )
            .execute(&db_pool)
            .await;

            let reset_password_url = format!(
                "https://www.populist.us/auth/reset-password?token={}",
                reset_token
            );

            // Send out email with link to reset new password
            EmailClient::default()
                .send_reset_password_email(email, reset_password_url)
                .await
                .expect("Failed to send reset password email");

            Ok(true)
        } else {
            Err(Error::EmailNotFound)
        }
    }

    async fn reset_password(
        &self,
        ctx: &Context<'_>,
        input: ResetPasswordInput,
    ) -> Result<bool, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        let update_result =
            User::reset_password(&db_pool, input.new_password, input.reset_token).await;

        if let Ok(user) = update_result {
            let email = user.email;
            EmailClient::default()
                .send_password_changed_email(email)
                .await
                .expect("Failed to send password changed email");

            Ok(true)
        } else {
            Err(Error::ResetTokenInvalid)
        }
    }

    async fn update_password(
        &self,
        ctx: &Context<'_>,
        input: UpdatePasswordInput,
    ) -> Result<bool, Error> {
        let user = ctx.data::<Option<TokenData<Claims>>>().unwrap();
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        match user {
            Some(user) => {
                let user_pw_result = sqlx::query!(
                    r#"
            SELECT password FROM populist_user 
            WHERE id = $1"#,
                    user.claims.sub
                )
                .fetch_one(&db_pool)
                .await;

                if let Ok(user_pw) = user_pw_result {
                    let password_is_valid = bcrypt::verify(input.old_password, &user_pw.password);

                    if password_is_valid {
                        let update_result =
                            User::update_password(&db_pool, input.new_password, user.claims.sub)
                                .await;
                        if update_result.is_ok() {
                            EmailClient::default()
                                .send_password_changed_email(user.claims.email.clone())
                                .await
                                .expect("Failed to send password changed email");
                            Ok(true)
                        } else {
                            Err(Error::ResetTokenInvalid)
                        }
                    } else {
                        Err(Error::PasswordError)
                    }
                } else {
                    Err(Error::EmailOrUsernameNotFound)
                }
            }
            None => Err(Error::Unauthorized),
        }
    }
}
