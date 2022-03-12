use crate::{
    context::ApiContext,
    mutation::StaffOnly,
    types::{CreateUserResult, Error, LoginResult, UserResult},
};
use async_graphql::*;
use auth::{create_access_token_for_user, create_random_token, create_temporary_username};
use db::{Address, CreateUserInput, CreateUserWithProfileInput, User};
use mailers::EmailPrototype;
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
    email: String,
    password: String,
    confirm_password: String,
    reset_token: String,
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

                // Send out email with username, temp password, and link to confirm & set password
                let prototype = EmailPrototype {
                    recipient: input.email,
                    subject: "Welcome to Populist".to_string(),
                    template_id: None,
                    template_data: Some(format!(
                        r#"
                Welcome to Populist!

                Thanks for creating an account with us!  Visit the link below to to verify your email address.

                {}
            "#,
                        account_confirmation_url
                    )),
                };

                mailers::EmailClient::send_mail(prototype)
                    .await
                    .expect("Something went wrong sending out a new user email");

                Ok(LoginResult { access_token })
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
            AND confirmed_at = NULL
        "#,
            confirmation_token
        )
        .execute(&db_pool)
        .await;

        if let Ok(_confirmed) = confirmed_user_result {
            Ok(true)
        } else {
            Err(Error::ConfirmationError)
        }
    }

    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResult, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let user_lookup = User::find_by_email_or_username(&db_pool, input.email_or_username).await;

        if let Ok(user) = user_lookup {
            let password_is_valid = bcrypt::verify(input.password, &user.password);

            if password_is_valid {
                let token = create_access_token_for_user(user)?;
                Ok(LoginResult {
                    access_token: token,
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
            let prototype = EmailPrototype {
                recipient: email,
                subject: "Reset your Password".to_string(),
                template_id: None,
                template_data: Some(format!(
                    r#"
                Lets get you setup with a new password.

                Visit the link below to to setup a new password.

                {}
            "#,
                    reset_password_url
                )),
            };

            mailers::EmailClient::send_mail(prototype)
                .await
                .expect("Something went wrong sending out a new user email");

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
        if input.password != input.confirm_password {
            return Err(Error::PasswordsDoNotMatch);
        };

        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        let update_result =
            User::update_password(&db_pool, input.password, input.reset_token).await;

        if let Ok(_) = update_result {
            // Send out email with confirming password has been changed, link to login
            let prototype = EmailPrototype {
                recipient: input.email.clone(),
                subject: "Reset your Password".to_string(),
                template_id: None,
                template_data: Some(format!(
                    r#"
                Your password has changed.  If you did this, just ignore this email.  If else, contact us right away at <info@populist.us>

                Click here to sign in.
            "#,
                )),
            };

            mailers::EmailClient::send_mail(prototype)
                .await
                .expect("Something went wrong sending out a new user email");

            Ok(true)
        } else {
            Err(Error::ResetTokenInvalid)
        }
    }
}
