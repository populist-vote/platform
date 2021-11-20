use crate::DateTime;
use async_graphql::{Enum, InputObject};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use strum_macros::Display;

#[derive(Enum, Debug, Display, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "author_type", rename_all = "lowercase")]
pub enum AuthorType {
    USER,
    POLITICIAN,
    ORGANIZATION,
    OTHER,
}

#[derive(Enum, Display, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "argument_position", rename_all = "lowercase")]
pub enum ArgumentPosition {
    SUPPORT,
    NEUTRAL,
    OPPOSE,
}

#[derive(FromRow, Debug, Clone)]
pub struct Argument {
    pub id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub title: String,
    pub position: ArgumentPosition,
    pub body: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct CreateArgumentInput {
    pub title: String,
    pub author_id: String,
    pub position: ArgumentPosition,
    pub body: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateArgumentInput {
    pub title: Option<String>,
    pub position: ArgumentPosition,
    pub body: Option<String>,
}

impl Argument {
    pub async fn create(
        db_pool: &PgPool,
        author_id: uuid::Uuid,
        input: &CreateArgumentInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Argument,
            r#"
                INSERT INTO argument (title, author_id, position, body) 
                VALUES ($1, $2, $3, $4) 
                RETURNING id, title, author_id, position AS "position:ArgumentPosition", body, created_at, updated_at
            "#,
            input.title,
            author_id,
            input.position as ArgumentPosition,
            input.body,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateArgumentInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Argument,
            r#"
                UPDATE argument
                SET title = COALESCE($2, title),
                    position = COALESCE($3, position),
                    body = COALESCE($4, body)
                WHERE id=$1    
                RETURNING id, title, author_id, position AS "position:ArgumentPosition", body, created_at, updated_at
            "#,
            id,
            input.title,
            input.position as ArgumentPosition,
            input.body,
        ).fetch_one(db_pool).await?;
        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM argument WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }
}
