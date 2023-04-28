use sqlx::FromRow;

use crate::DateTime;

#[derive(FromRow, Debug, Clone)]
pub struct Respondent {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

pub struct UpsertRespondentInput {
    pub id: Option<uuid::Uuid>,
    pub name: String,
    pub email: String,
}

impl Respondent {
    pub async fn upsert(
        pool: &sqlx::PgPool,
        input: &UpsertRespondentInput,
    ) -> Result<Respondent, sqlx::Error> {
        let respondent = sqlx::query_as!(
            Respondent,
            r#"
            INSERT INTO respondent (
                id,
                name,
                email
            ) VALUES (
                $1,
                $2,
                $3
            ) ON CONFLICT (id) DO UPDATE SET
                name = $1,
                updated_at = now()
            RETURNING *
            "#,
            input.id,
            input.name,
            input.email
        )
        .fetch_one(pool)
        .await?;

        Ok(respondent)
    }

    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
    ) -> Result<Option<Respondent>, sqlx::Error> {
        let respondent = sqlx::query_as!(
            Respondent,
            r#"
            SELECT * FROM respondent WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(respondent)
    }
}