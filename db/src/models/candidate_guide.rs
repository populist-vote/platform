use async_graphql::InputObject;
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

use crate::DateTime;

#[derive(FromRow, Debug, Clone)]
pub struct CandidateGuide {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: Option<String>,
    pub race_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject, Debug)]
pub struct UpsertCandidateGuideInput {
    pub id: Option<Uuid>,
    pub name: Option<String>,
}

impl CandidateGuide {
    pub async fn upsert(
        db_pool: &PgPool,
        input: &UpsertCandidateGuideInput,
    ) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(uuid::Uuid::new_v4);
        let record = sqlx::query_as!(
            CandidateGuide,
            r#"
                INSERT INTO candidate_guide
                (id, name)
                VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE SET
                    name = COALESCE($2, candidate_guide.name)
                RETURNING id, name, created_at, created_by, updated_at, organization_id, race_id
            "#,
            id,
            input.name,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                DELETE FROM candidate_guide
                WHERE id = $1
            "#,
            id,
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }
}
