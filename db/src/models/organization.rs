use crate::DateTime;
use crate::IssueTag;
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

static _ORGANIZATION_COLUMNS: &'static str = "id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, headquarters_phone, tax_classification, created_at, updated_at";

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

    pub async fn issue_tags(
        db_pool: &PgPool,
        organization_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, it.created_at, it.updated_at FROM issue_tag it
                JOIN organization_issue_tags
                ON organization_issue_tags.issue_tag_id = it.id
                WHERE organization_issue_tags.organization_id = $1
            "#,
            organization_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records.into())
    }
}
