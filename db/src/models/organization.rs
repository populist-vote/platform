use crate::{
    models::{issue_tag::IssueTag, user::User},
    DateTime,
};
use async_graphql::InputObject;
use slugify::slugify;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Organization {
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub email: Option<String>,
    pub headquarters_phone: Option<String>,
    pub tax_classification: Option<String>,
    // pub issue_tags: Vec<IssueTag>,
    // pub created_by: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct CreateOrganizationInput {
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateOrganizationInput {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
}

#[derive(InputObject)]
pub struct OrganizationSearch {
    name: Option<String>,
}

impl Organization {
    pub async fn create(
        db_pool: &PgPool,
        input: &CreateOrganizationInput,
    ) -> Result<Self, sqlx::Error> {
        let slug = slugify!(&input.name); // TODO run a query and ensure this is Unique
        let record = sqlx::query_as!(
            Organization,
            "INSERT INTO organization (slug, name, description, thumbnail_image_url, website_url) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING *",
            slug,
            input.name,
            input.description,
            input.thumbnail_image_url,
            input.website_url
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateOrganizationInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Organization,
            "UPDATE organization
            SET slug = COALESCE($2, slug),
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                thumbnail_image_url = COALESCE($5, thumbnail_image_url),
                website_url = COALESCE($6, website_url)
            WHERE id=$1
            RETURNING *",
            id,
            input.slug,
            input.name,
            input.description,
            input.thumbnail_image_url,
            input.website_url
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM organization WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(Organization, "SELECT * FROM organization")
            .fetch_all(db_pool)
            .await?;
        Ok(records.into())
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &OrganizationSearch,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Organization,
            "SELECT * FROM organization
             WHERE $1::text IS NULL OR levenshtein($1, name) <=5",
            search.name
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records.into())
    }
}
