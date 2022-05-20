use sqlx::{FromRow, PgPool};

use crate::DateTime;

#[derive(FromRow, Debug, Clone)]
pub struct VotingGuide {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl VotingGuide {
    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            VotingGuide,
            r#"
                SELECT
                    id,
                    user_id,
                    title,
                    description,
                    created_at,
                    updated_at
                FROM
                    voting_guide
                WHERE
                    id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }
}
