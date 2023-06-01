use async_graphql::{Enum, InputObject};
use serde_json::Value as JSON;
use sqlx::FromRow;
use strum_macros::Display;

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug, Display, sqlx::Type)]
#[sqlx(type_name = "embed_type", rename_all = "lowercase")]
pub enum EmbedType {
    Legislation,
    Politician,
    Question,
    Poll,
}

#[derive(FromRow, Debug, Clone)]
pub struct Embed {
    pub id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub populist_url: String,
    pub attributes: JSON,
    pub embed_type: EmbedType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: uuid::Uuid,
    pub updated_by: uuid::Uuid,
}

#[derive(InputObject)]
pub struct UpsertEmbedInput {
    pub id: Option<uuid::Uuid>,
    pub organization_id: Option<uuid::Uuid>,
    pub embed_type: Option<EmbedType>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub populist_url: Option<String>,
    pub attributes: Option<JSON>,
}

#[derive(InputObject, Debug, Default)]
pub struct EmbedFilter {
    pub embed_type: Option<EmbedType>,
}

impl Embed {
    pub async fn upsert(
        pool: &sqlx::PgPool,
        input: &UpsertEmbedInput,
        updated_by: &uuid::Uuid,
    ) -> Result<Embed, sqlx::Error> {
        let created_by = updated_by;
        let embed = sqlx::query_as!(
            Embed,
            r#"
            INSERT INTO embed (
                id,
                organization_id,
                name,
                description,
                populist_url,
                embed_type,
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
                $8,
                $9
            )
            ON CONFLICT (id) DO UPDATE SET
                organization_id = $2,
                name = $3,
                description = $4,
                populist_url = $5,
                embed_type = $6,
                attributes = $7,
                updated_by = $9
            RETURNING id,
                organization_id,
                name,
                description,
                populist_url,
                embed_type AS "embed_type:EmbedType",
                attributes,
                created_by,
                updated_by,
                created_at,
                updated_at
            "#,
            input.id.unwrap_or(uuid::Uuid::new_v4()),
            input.organization_id,
            input.name,
            input.description,
            input.populist_url,
            input.embed_type as Option<EmbedType>,
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
            SELECT id,
                organization_id,
                name,
                description,
                populist_url,
                embed_type AS "embed_type:EmbedType",
                attributes,
                created_by,
                updated_by,
                created_at,
                updated_at
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
        filter: EmbedFilter,
    ) -> Result<Vec<Embed>, sqlx::Error> {
        let embeds = sqlx::query_as!(
            Embed,
            r#"
            SELECT id,
                organization_id,
                name,
                description,
                populist_url,
                embed_type AS "embed_type:EmbedType",
                attributes,
                created_by,
                updated_by,
                created_at,
                updated_at
            FROM embed
            WHERE organization_id = $1
            AND ($2::embed_type IS NULL OR embed_type = $2)
            ORDER BY updated_at DESC
            "#,
            organization_id,
            filter.embed_type as Option<EmbedType>
        )
        .fetch_all(pool)
        .await?;

        Ok(embeds)
    }
}
