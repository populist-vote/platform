use crate::DateTime;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub const MAX_ACTIVE_API_KEYS: i64 = 10;

#[derive(Debug, Clone, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub last_used_at: Option<DateTime>,
    pub revoked_at: Option<DateTime>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiKeyError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("API keys can only be created for confirmed users")]
    UserNotConfirmed,

    #[error("You can have at most {MAX_ACTIVE_API_KEYS} active API keys")]
    ActiveKeyLimit,

    #[error("An active API key with this name already exists")]
    DuplicateName,
}

impl ApiKey {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        name: &str,
        key_prefix: &str,
        key_hash: &[u8],
    ) -> Result<Self, ApiKeyError> {
        let mut transaction = pool.begin().await?;

        // Lock the user row so concurrent key creation cannot bypass the per-user limit.
        let confirmed_at = sqlx::query_scalar::<_, Option<DateTime>>(
            "SELECT confirmed_at FROM populist_user WHERE id = $1 FOR UPDATE",
        )
        .bind(user_id)
        .fetch_optional(&mut *transaction)
        .await?
        .flatten();

        if confirmed_at.is_none() {
            return Err(ApiKeyError::UserNotConfirmed);
        }

        let active_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_api_key WHERE user_id = $1 AND revoked_at IS NULL",
        )
        .bind(user_id)
        .fetch_one(&mut *transaction)
        .await?;

        if active_count >= MAX_ACTIVE_API_KEYS {
            return Err(ApiKeyError::ActiveKeyLimit);
        }

        let key = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO user_api_key (user_id, name, key_prefix, key_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, name, key_prefix, last_used_at, revoked_at,
                      created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(key_prefix)
        .bind(key_hash)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| match &error {
            sqlx::Error::Database(database_error)
                if database_error.constraint() == Some("user_api_key_active_name_idx") =>
            {
                ApiKeyError::DuplicateName
            }
            _ => ApiKeyError::Database(error),
        })?;

        transaction.commit().await?;
        Ok(key)
    }

    pub async fn list_active(pool: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, user_id, name, key_prefix, last_used_at, revoked_at,
                   created_at, updated_at
            FROM user_api_key
            WHERE user_id = $1 AND revoked_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_active_by_hash(
        pool: &PgPool,
        key_hash: &[u8],
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, user_id, name, key_prefix, last_used_at, revoked_at,
                   created_at, updated_at
            FROM user_api_key
            WHERE key_hash = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(key_hash)
        .fetch_optional(pool)
        .await
    }

    pub async fn mark_used(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        // Avoid a database write on every request while keeping last-used data useful.
        sqlx::query(
            r#"
            UPDATE user_api_key
            SET last_used_at = NOW(), updated_at = NOW()
            WHERE id = $1
              AND (last_used_at IS NULL OR last_used_at < NOW() - INTERVAL '5 minutes')
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn revoke(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE user_api_key
            SET revoked_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
            RETURNING id, user_id, name, key_prefix, last_used_at, revoked_at,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }
}
