use crate::{
    context::ApiContext,
    is_admin,
    mutation::StaffOnly,
    types::{CreateUserResult, Error, LoginResult},
};
use async_graphql::*;
use auth::{create_access_token_for_user, create_random_token, create_temporary_username, Claims};
use db::{AddressInput, Coordinates, CreateUserInput, CreateUserWithProfileInput, User};
use geocodio::GeocodioProxy;
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
    #[graphql(validator(email))]
    pub email: String,
    #[graphql(validator(min_length = 8))]
    pub password: String,
    pub address: AddressInput,
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
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: CreateUserInput,
    ) -> Result<CreateUserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = User::create(&db_pool, &input).await?;

        Ok(CreateUserResult::from(new_record))
    }

    #[graphql(visible = "is_admin")]
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

        // Use geocodio to get congressional district and state legislative districts
        let address_clone = input.address.clone();
        let geocodio = GeocodioProxy::new().unwrap();
        let geocode_result = geocodio
            .geocode(
                geocodio::AddressParams::AddressInput(geocodio::AddressInput {
                    line_1: address_clone.line_1,
                    line_2: address_clone.line_2,
                    city: address_clone.city,
                    state: address_clone.state.to_string(),
                    country: address_clone.country,
                    postal_code: address_clone.postal_code,
                }),
                Some(&["cd", "stateleg"]),
            )
            .await;

        match geocode_result {
            Ok(geocodio_data) => {
                let coordinates = geocodio_data.results[0].location.clone();
                let county = geocodio_data.results[0].address_components.county.clone();
                let primary_result = geocodio_data.results[0].fields.as_ref().unwrap();
                let congressional_district =
                    primary_result.congressional_districts.as_ref().unwrap()[0].district_number;
                let state_legislative_districts =
                    primary_result.state_legislative_districts.as_ref().unwrap();
                let state_house_district = &state_legislative_districts.house[0].district_number;
                let state_senate_district = &state_legislative_districts.senate[0].district_number;

                let new_user_input = CreateUserWithProfileInput {
                    email: input.email.clone(),
                    username: temp_username,
                    password: input.password,
                    address: AddressInput {
                        coordinates: Some(Coordinates {
                            latitude: coordinates.latitude,
                            longitude: coordinates.longitude,
                        }),
                        county,
                        congressional_district: Some(congressional_district.into()),
                        state_house_district: Some(state_house_district.parse::<i32>().unwrap()),
                        state_senate_district: Some(state_senate_district.parse::<i32>().unwrap()),
                        ..input.address
                    },
                    confirmation_token: confirmation_token.clone(),
                };

                let new_record = User::create_with_profile(&db_pool, &new_user_input).await;

                match new_record {
                    Ok(new_user) => {
                        // Create Access Token to log user in
                        let access_token = create_access_token_for_user(new_user.clone())?;

                        let account_confirmation_url = format!(
                            "{}/auth/confirm?token={}",
                            config::Config::default().web_app_url,
                            confirmation_token
                        );

                        if let Err(err) = EmailClient::default()
                            .send_welcome_email(new_user.email, account_confirmation_url)
                            .await
                        {
                            println!("Error sending welcome email: {}", err)
                        }

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
            Err(err) => Err(err.into()),
        }
    }

    #[graphql(visible = "is_admin")]
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

    #[graphql(visible = "is_admin")]
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

    #[graphql(visible = "is_admin")]
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
                "{}/auth/password?token={}",
                config::Config::default().web_app_url,
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

    #[graphql(visible = "is_admin")]
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

    #[graphql(visible = "is_admin")]
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

    #[graphql(visible = "is_admin")]
    async fn logout(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        ctx.insert_http_header(
            SET_COOKIE,
            format!(
                "access_token=null; expires={}; Max-Age=0; HttpOnly; SameSite=None; Secure",
                (chrono::Utc::now() - chrono::Duration::hours(1)).format("%a, %d %b %Y %T GMT")
            ),
        );

        Ok(true)
    }
}
