use async_graphql::InputObject;
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

use crate::DateTime;

#[derive(FromRow, Debug, Clone)]
pub struct CandidateGuide {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject, Debug)]
pub struct UpsertCandidateGuideInput {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub organization_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub race_ids: Option<Vec<Uuid>>,
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
                (id, name, organization_id, created_by)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (id) DO UPDATE SET
                    name = COALESCE($2, candidate_guide.name)
                RETURNING id, name, created_at, created_by, updated_at, organization_id
            "#,
            id,
            input.name,
            input.organization_id,
            input.user_id,
        )
        .fetch_one(db_pool)
        .await?;

        if let Some(race_ids) = &input.race_ids {
            if !race_ids.is_empty() {
                sqlx::query!(
                    r#"
                        INSERT INTO candidate_guide_races (candidate_guide_id, race_id)
                        SELECT $1, unnest($2::uuid[])
                    "#,
                    id,
                    race_ids,
                )
                .execute(db_pool)
                .await?;
            }
        }

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                WITH deleted_guide AS (
                    DELETE FROM candidate_guide
                    WHERE id = $1
                )
                DELETE FROM embed
                WHERE attributes->>'candidate_guide_id' = $1::text
            "#,
            id,
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(db_pool: &PgPool, id: Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            CandidateGuide,
            r#"
                SELECT
                    id,
                    name,
                    created_at,
                    created_by,
                    updated_at,
                    organization_id
                FROM candidate_guide
                WHERE id = $1
            "#,
            id,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_organization(
        db_pool: &PgPool,
        organization_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            CandidateGuide,
            r#"
                SELECT
                    id,
                    name,
                    created_at,
                    created_by,
                    updated_at,
                    organization_id
                FROM candidate_guide
                WHERE organization_id = $1
            "#,
            organization_id,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
