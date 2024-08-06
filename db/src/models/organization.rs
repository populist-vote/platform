use crate::CreateOrConnectIssueTagInput;
use crate::DateTime;
use crate::IssueTag;
use crate::IssueTagIdentifier;
use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSON;
use slugify::slugify;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone, Eq, PartialEq)]
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
    /// Organization for a politician's campaign
    pub politician_id: Option<uuid::Uuid>,
    pub assets: JSON,
    pub attributes: JSON,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[serde_with::serde_as]
#[derive(InputObject, Debug, Default, Serialize, Deserialize)]
pub struct CreateOrganizationInput {
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
    pub assets: Option<serde_json::Value>,
}

#[serde_with::serde_as]
#[derive(InputObject, Debug, Default, Serialize, Deserialize)]
pub struct UpdateOrganizationInput {
    pub id: uuid::Uuid,
    pub slug: Option<String>,
    pub name: Option<String>,
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
    pub assets: Option<serde_json::Value>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
}

#[derive(Default, InputObject)]
pub struct OrganizationSearch {
    name: Option<String>,
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
        let id = uuid::Uuid::new_v4();
        let slug = slugify!(&input.name);

        let record = sqlx::query_as!(
            Organization,
            r#"
                INSERT INTO organization (id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, votesmart_sig_id, headquarters_address_id, headquarters_phone, tax_classification, assets)
                VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                RETURNING
                    id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, votesmart_sig_id, headquarters_address_id, headquarters_phone, tax_classification, politician_id, assets, attributes, created_at, updated_at
            "#,
            id,
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
            input.tax_classification,
            input.assets
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        input: &UpdateOrganizationInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Organization,
            r#"
                UPDATE organization SET
                    slug = COALESCE($1, slug),
                    name = COALESCE($2, name),
                    description = COALESCE($3, description),
                    thumbnail_image_url = COALESCE($4, thumbnail_image_url),
                    website_url = COALESCE($5, website_url),
                    facebook_url = COALESCE($6, facebook_url),
                    twitter_url = COALESCE($7, twitter_url),
                    instagram_url = COALESCE($8, instagram_url),
                    email = COALESCE($9, email),
                    votesmart_sig_id = COALESCE($10, votesmart_sig_id),
                    headquarters_address_id = COALESCE($11, headquarters_address_id),
                    headquarters_phone = COALESCE($12, headquarters_phone),
                    tax_classification = COALESCE($13, tax_classification),
                    assets = COALESCE($14, assets)
                WHERE id = $15
                RETURNING
                    id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, votesmart_sig_id, headquarters_address_id, headquarters_phone, tax_classification, politician_id, assets, attributes, created_at, updated_at
            "#,
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
            input.tax_classification,
            input.assets,
            input.id
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
