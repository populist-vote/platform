use crate::DateTime;
use async_graphql::InputObject;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Respondent {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, InputObject)]
pub struct UpsertRespondentInput {
    pub id: Option<uuid::Uuid>,
    pub name: String,
    pub email: String,
    pub organization_id: uuid::Uuid,
}

impl Respondent {
    pub async fn upsert(
        pool: &sqlx::PgPool,
        input: &UpsertRespondentInput,
    ) -> Result<Respondent, sqlx::Error> {
        let id = input.id.unwrap_or_else(uuid::Uuid::new_v4);
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
            ) ON CONFLICT (email) DO UPDATE SET
                name = $2,
                updated_at = now()
            RETURNING *
            "#,
            id,
            input.name,
            input.email
        )
        .fetch_one(pool)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO organization_respondents (
                organization_id,
                respondent_id
            ) VALUES (
                $1,
                $2
            ) ON CONFLICT (organization_id, respondent_id) DO NOTHING
            "#,
            input.organization_id,
            respondent.id
        )
        .execute(pool)
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
