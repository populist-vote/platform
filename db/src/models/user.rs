use async_graphql::{validators::Email, InputObject};
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};

use crate::{DateTime, Error};

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

#[derive(InputObject)]
pub struct CreateUserInput {
    #[graphql(validator(Email))]
    email: String,
    username: String,
    password: String,
}

#[derive(Debug, Clone, strum_macros::Display, Type, Serialize, Deserialize)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum Role {
    SUPERUSER,
    STAFF,
    PREMIUM,
    BASIC,
}

impl User {
    pub async fn create(db_pool: &PgPool, input: &CreateUserInput) -> Result<Self, Error> {
        let hash = bcrypt::hash(&input.password).unwrap();
        let record = sqlx::query_as!(
            User,
            r#"
                INSERT INTO populist_user (email, username, password)
                VALUES ($1, $2, $3)
                RETURNING id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at
            "#,
            input.email,
            input.username,
            hash,
        ).fetch_one(db_pool).await?;

        Ok(record)
    }

    pub async fn find_by_email_or_username(
        db_pool: &PgPool,
        email_or_username: String,
    ) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at FROM populist_user 
                WHERE $1 IN(email, username);
            "#,
            email_or_username
        ).fetch_optional(db_pool).await?;

        match record {
            Some(record) => Ok(record),
            None => Err(Error::EmailOrUsernameNotFound),
        }
    }
}
