use crate::CreateOrConnectIssueTagInput;
use crate::DateTime;
use crate::IssueTag;
use crate::IssueTagIdentifier;

use async_graphql::InputObject;

use serde::{Deserialize, Serialize};
use slugify::slugify;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone, Eq, PartialEq, Hash)]
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
    pub votesmart_sig_id: Option<i32>,
    pub headquarters_address_id: Option<uuid::Uuid>,
    pub headquarters_phone: Option<String>,
    pub tax_classification: Option<String>,
    // pub created_by: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateOrganizationInput {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub email: Option<String>,
    pub votesmart_sig_id: Option<i32>,
    pub headquarters_address_id: Option<uuid::Uuid>,
    pub headquarters_phone: Option<String>,
    pub tax_classification: Option<String>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
}

#[derive(InputObject)]
pub struct UpdateOrganizationInput {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub email: Option<String>,
    pub votesmart_sig_id: Option<i32>,
    pub headquarters_address_id: Option<uuid::Uuid>,
    pub headquarters_phone: Option<String>,
    pub tax_classification: Option<String>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
}

#[derive(InputObject)]
pub struct OrganizationSearch {
    name: Option<String>,
}

impl Default for OrganizationSearch {
    fn default() -> Self {
        OrganizationSearch { name: None }
    }
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateOrConnectOrganizationInput {
    pub create: Option<Vec<CreateOrganizationInput>>,
    pub connect: Option<Vec<String>>, // Accept UUIDs or slugs
}

pub enum OrganizationIdentifier {
    Uuid(uuid::Uuid),
    Slug(String),
}

impl Organization {
    pub async fn create(
        db_pool: &PgPool,
        input: &CreateOrganizationInput,
    ) -> Result<Self, sqlx::Error> {
        let slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.name),
        }; // TODO run a query and ensure this is Unique
        let record = sqlx::query_as!(
            Organization,
            r#"
                WITH ins_author AS (
                    INSERT INTO author (author_type) VALUES ('organization')
                    ON CONFLICT DO NOTHING
                    RETURNING id AS author_id
                ),
                o AS (
                    INSERT INTO organization (id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, votesmart_sig_id, headquarters_address_id, headquarters_phone, tax_classification) 
                    VALUES ((SELECT author_id FROM ins_author), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                    RETURNING id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, votesmart_sig_id, headquarters_address_id, headquarters_phone, tax_classification, created_at, updated_at
                )
                SELECT o.* FROM o
            "#,
            slug,
            input.name,
            input.description,
            input.thumbnail_image_url,
            input.website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.email,
            input.votesmart_sig_id,
            input.headquarters_address_id,
            input.headquarters_phone,
            input.tax_classification
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
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
                website_url = COALESCE($6, website_url),
                facebook_url = COALESCE($7, facebook_url),
                twitter_url = COALESCE($8, twitter_url),
                instagram_url = COALESCE($9, instagram_url),
                email = COALESCE($10, email),
                votesmart_sig_id = COALESCE($11, votesmart_sig_id),
                headquarters_address_id = COALESCE($12, headquarters_address_id),
                headquarters_phone = COALESCE($13, headquarters_phone),
                tax_classification = COALESCE($14, tax_classification)
            WHERE id=$1
            RETURNING *",
            id,
            input.slug,
            input.name,
            input.description,
            input.thumbnail_image_url,
            input.website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.email,
            input.votesmart_sig_id,
            input.headquarters_address_id,
            input.headquarters_phone,
            input.tax_classification
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
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
        Ok(records)
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &OrganizationSearch,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Organization,
            r#"
                SELECT * FROM organization
                WHERE ($1::text IS NULL OR levenshtein($1, name) <=5)
             "#,
            search.name
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Organization,
            r#"
                SELECT * FROM organization
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
            Organization,
            r#"
                SELECT * FROM organization
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn connect_issue_tag(
        db_pool: &PgPool,
        organization_id: uuid::Uuid,
        issue_tag_identifier: IssueTagIdentifier,
    ) -> Result<(), sqlx::Error> {
        match issue_tag_identifier {
            IssueTagIdentifier::Uuid(issue_tag_id) => {
                sqlx::query_as!(
                    Organization,
                    r#"
                        INSERT INTO organization_issue_tags (organization_id, issue_tag_id) 
                        VALUES ($1, $2)
                    "#,
                    organization_id,
                    issue_tag_id
                )
                .execute(db_pool)
                .await?;
            }
            IssueTagIdentifier::Slug(issue_tag_slug) => {
                sqlx::query_as!(
                    Organization,
                    r#"
                        INSERT INTO organization_issue_tags (organization_id, issue_tag_id) 
                        VALUES ($1, (SELECT id FROM issue_tag WHERE slug = $2))
                    "#,
                    organization_id,
                    issue_tag_slug
                )
                .execute(db_pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        organization_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN organization_issue_tags
                ON organization_issue_tags.issue_tag_id = it.id
                WHERE organization_issue_tags.organization_id = $1
            "#,
            organization_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
