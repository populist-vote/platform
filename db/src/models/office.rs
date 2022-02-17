use super::enums::{PoliticalScope, State};
use crate::DateTime;
use async_graphql::{InputObject, ID};
use serde::{Deserialize, Serialize};
use slugify::slugify;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Office {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub office_type: Option<String>,
    pub political_scope: PoliticalScope,
    pub state: Option<State>,
    pub encumbent_id: uuid::Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateOfficeInput {
    pub slug: Option<String>,
    pub title: String,
    pub office_type: Option<String>,
    pub political_scope: PoliticalScope,
    pub state: Option<State>,
    pub encumbent_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdateOfficeInput {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub office_type: Option<String>,
    pub political_scope: Option<PoliticalScope>,
    pub state: Option<State>,
    pub encumbent_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct OfficeSearch {
    query: Option<String>,
    state: Option<State>,
}

impl Default for OfficeSearch {
    fn default() -> Self {
        Self {
            query: None,
            state: None,
        }
    }
}

impl Office {
    pub async fn create(db_pool: &PgPool, input: &CreateOfficeInput) -> Result<Self, sqlx::Error> {
        let slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.title),
        };

        let record = sqlx::query_as!(
            Office,
            r#"
                INSERT INTO office (slug, title, political_scope, state, encumbent_id)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, slug, title, office_type, political_scope AS "political_scope:PoliticalScope", encumbent_id, state AS "state:State", created_at, updated_at
            "#,
            slug,
            input.title,
            input.political_scope as PoliticalScope,
            input.state as Option<State>,
            input.encumbent_id,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateOfficeInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Office,
            r#"
                UPDATE office
                SET slug = COALESCE($2, slug), 
                    title = COALESCE($3, title), 
                    political_scope = COALESCE($4, political_scope),
                    state = COALESCE($5, state),
                    encumbent_id = COALESCE($6, encumbent_id)
                WHERE id = $1
                RETURNING id, slug, title, office_type, political_scope AS "political_scope:PoliticalScope", encumbent_id, state AS "state:State", created_at, updated_at
            "#,
            id,
            input.slug,
            input.title,
            input.political_scope as Option<PoliticalScope>,
            input.state as Option<State>,
            input.encumbent_id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM office WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Office,
            r#"
                SELECT id, slug, title, office_type, political_scope AS "political_scope:PoliticalScope", encumbent_id, state AS "state:State", created_at, updated_at FROM office
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_slug(db_pool: &PgPool, slug: String) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Office,
            r#"
                SELECT id, slug, title, office_type, political_scope AS "political_scope:PoliticalScope", encumbent_id, state AS "state:State", created_at, updated_at FROM office
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn search(db_pool: &PgPool, input: &OfficeSearch) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Office,
            r#"
                SELECT id, slug, title, office_type, political_scope AS "political_scope:PoliticalScope", encumbent_id, state AS "state:State", created_at, updated_at FROM office
                WHERE (($1::text = '') IS NOT FALSE OR to_tsvector(concat_ws(' ', slug, title)) @@ to_tsquery($1))
                AND ($2::state IS NULL OR state = $2)
                
            "#,
            input.query,
            input.state as Option<State>,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
