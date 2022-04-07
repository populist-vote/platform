use async_graphql::{Enum, InputObject};
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};

use crate::{DateTime, Error};

use super::enums::State;

#[derive(FromRow, Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub password: String,
    pub role: Role,
    pub created_at: DateTime,
    pub confirmed_at: Option<DateTime>,
    pub updated_at: DateTime,
}

#[derive(FromRow, Debug, Clone)]
pub struct UserProfile {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub address_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct Address {
    pub id: uuid::Uuid,
    pub line_1: String,
    pub line_2: Option<String>,
    pub city: String,
    pub state: State,
    pub country: String,
    pub postal_code: String,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct AddressInput {
    pub line_1: String,
    pub line_2: Option<String>,
    pub city: String,
    pub state: State,
    pub country: String,
    pub postal_code: String,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct CreateUserInput {
    #[graphql(validator(email))]
    pub email: String,
    pub username: String,
    pub password: String,
    pub role: Option<Role>,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct CreateUserWithProfileInput {
    #[graphql(validator(email))]
    pub email: String,
    pub username: String,
    pub password: String,
    pub address: AddressInput,
    pub confirmation_token: String,
}

#[derive(
    Debug, Clone, strum_macros::Display, Type, Serialize, Deserialize, Copy, Eq, PartialEq, Enum,
)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    SUPERUSER,
    STAFF,
    PREMIUM,
    BASIC,
}

impl User {
    pub async fn create(db_pool: &PgPool, input: &CreateUserInput) -> Result<Self, Error> {
        let hash = bcrypt::hash(&input.password).unwrap();
        let role = input.role.unwrap_or(Role::BASIC);
        let record = sqlx::query_as!(
            User,
            r#"
                INSERT INTO populist_user (email, username, password, role)
                VALUES (LOWER($1), LOWER($2), $3, $4)
                RETURNING id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at
            "#,
            input.email,
            input.username,
            hash,
            role as Role
        ).fetch_one(db_pool).await?;

        Ok(record)
    }

    pub async fn create_with_profile(
        db_pool: &PgPool,
        input: &CreateUserWithProfileInput,
    ) -> Result<Self, Error> {
        let hash = bcrypt::hash(&input.password).unwrap();
        let record = sqlx::query_as!(
            User,
            r#"
                WITH ins_user AS (
                    INSERT INTO populist_user (email, username, password, role, confirmation_token)
                    VALUES (LOWER($1), LOWER($2), $3, $4, $11)
                    RETURNING id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at
                ),
                ins_address AS (
                    INSERT INTO address (line_1, line_2, city, state, country, postal_code)
                    VALUES ($5, $6, $7, $8, $9, $10)
                    RETURNING id
                ),
                ins_profile AS (
                    INSERT INTO user_profile (address_id, user_id)
                    VALUES ((SELECT id FROM ins_address), (SELECT id FROM ins_user))
                )
                SELECT ins_user.* FROM ins_user
            "#,
            input.email,
            input.username,
            hash,
            Role::BASIC as Role,
            input.address.line_1,
            input.address.line_2,
            input.address.city,
            input.address.state.to_string(),
            input.address.country,
            input.address.postal_code,
            input.confirmation_token
        ).fetch_one(db_pool).await?;

        // Need to handle case of existing user

        Ok(record)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at FROM populist_user 
                WHERE $1 = id;
            "#,
            id
        ).fetch_optional(db_pool).await?;

        match record {
            Some(record) => Ok(record),
            None => Err(Error::EmailOrUsernameNotFound),
        }
    }

    pub async fn find_by_email_or_username(
        db_pool: &PgPool,
        email_or_username: String,
    ) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            User,
            r#"
                SELECT 
                    id, 
                    email, 
                    username, 
                    password, 
                    role AS "role:Role", 
                    created_at, 
                    confirmed_at, 
                    updated_at 
                FROM populist_user 
                WHERE LOWER($1) IN(email, username);
            "#,
            email_or_username
        )
        .fetch_optional(db_pool)
        .await?;

        match record {
            Some(record) => Ok(record),
            None => Err(Error::EmailOrUsernameNotFound),
        }
    }

    pub async fn validate_email_exists(db_pool: &PgPool, email: String) -> Result<bool, Error> {
        let existing_user = sqlx::query!(
            r#"
            SELECT id FROM populist_user WHERE email = LOWER($1)
        "#,
            email
        )
        .fetch_optional(db_pool)
        .await?;

        if let Some(_user) = existing_user {
            Ok(true)
        } else {
            Err(Error::EmailOrUsernameNotFound)
        }
    }

    pub async fn reset_password(
        db_pool: &PgPool,
        new_password: String,
        reset_token: String,
    ) -> Result<Self, Error> {
        let hash = bcrypt::hash(&new_password).unwrap();

        let update_result = sqlx::query_as!(User,
            r#"
                UPDATE populist_user
                SET password = $1,
                    reset_token = NULL
                WHERE reset_token = $2
                AND reset_token_expires_at > now()
                RETURNING id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at
            "#,
            hash,
            reset_token
        )
        .fetch_optional(db_pool)
        .await;

        if let Ok(Some(user)) = update_result {
            Ok(user)
        } else {
            Err(Error::ResetTokenInvalid)
        }
    }

    pub async fn update_password(
        db_pool: &PgPool,
        new_password: String,
        user_id: uuid::Uuid,
    ) -> Result<bool, Error> {
        let hash = bcrypt::hash(&new_password).unwrap();

        let update_result = sqlx::query!(
            r#"
                UPDATE populist_user
                SET password = $1
                WHERE id = $2
            "#,
            hash,
            user_id
        )
        .execute(db_pool)
        .await;

        if update_result.is_ok() {
            Ok(true)
        } else {
            Err(Error::Custom(
                "Your password could not be updated".to_string(),
            ))
        }
    }
}
