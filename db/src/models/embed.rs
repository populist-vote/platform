use async_graphql::InputObject;
use serde_json::Value as JSON;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Embed {
    pub id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub populist_url: String,
    pub attributes: JSON,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: uuid::Uuid,
    pub updated_by: uuid::Uuid,
}

#[derive(InputObject)]
pub struct UpsertEmbedInput {
    pub id: Option<uuid::Uuid>,
    pub organization_id: Option<uuid::Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub populist_url: Option<String>,
    pub attributes: Option<JSON>,
}

impl Embed {
    pub async fn upsert(
        pool: &sqlx::PgPool,
        input: &UpsertEmbedInput,
        updated_by: &uuid::Uuid,
    ) -> Result<Embed, sqlx::Error> {
        let created_by = updated_by.clone();
        let embed = sqlx::query_as!(
            Embed,
            r#"
            INSERT INTO embed (
                id,
                organization_id,
                name,
                description,
                populist_url,
                attributes,
                created_by,
                updated_by
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8
            )
            ON CONFLICT (id) DO UPDATE SET
                organization_id = $2,
                name = $3,
                description = $4,
                populist_url = $5,
                attributes = $6,
                updated_by = $8
            RETURNING *
            "#,
            input.id.unwrap_or(uuid::Uuid::new_v4()),
            input.organization_id,
            input.name,
            input.description,
            input.populist_url,
            input.attributes,
            created_by,
            updated_by
        )
        .fetch_one(pool)
        .await?;

        Ok(embed)
    }

    pub async fn delete(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM embed
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<Embed, sqlx::Error> {
        let embed = sqlx::query_as!(
            Embed,
            r#"
            SELECT *
            FROM embed
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(embed)
    }

    pub async fn find_by_organization_id(
        pool: &sqlx::PgPool,
        organization_id: uuid::Uuid,
    ) -> Result<Vec<Embed>, sqlx::Error> {
        let embeds = sqlx::query_as!(
            Embed,
            r#"
            SELECT *
            FROM embed
            WHERE organization_id = $1
            ORDER BY updated_at DESC
            "#,
            organization_id
        )
        .fetch_all(pool)
        .await?;

        Ok(embeds)
    }
}
