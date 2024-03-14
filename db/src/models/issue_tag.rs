use crate::{models::enums::State, DateTime, Organization, Politician};
use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct IssueTag {
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    // pub created_by: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject, Debug, Serialize, Deserialize)]
pub struct UpsertIssueTagInput {
    pub id: Option<uuid::Uuid>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(InputObject, Debug, Serialize, Deserialize)]
pub struct CreateOrConnectIssueTagInput {
    pub create: Option<Vec<UpsertIssueTagInput>>,
    pub connect: Option<Vec<String>>, //accepts UUIDs or slugs
}

#[derive(InputObject)]
pub struct IssueTagSearch {
    pub name: Option<String>,
}

pub enum IssueTagIdentifier {
    Uuid(uuid::Uuid),
    Slug(String),
}

impl IssueTag {
    pub async fn upsert(
        db_pool: &PgPool,
        input: &UpsertIssueTagInput,
    ) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(uuid::Uuid::new_v4);

        let record = sqlx::query_as!(
            IssueTag,
            r#"
                INSERT INTO issue_tag (id, slug, name, description, category) VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO UPDATE SET
                    slug = COALESCE($2, issue_tag.slug),   
                    name = COALESCE($3, issue_tag.name),
                    description = COALESCE($4, issue_tag.description),
                    category = COALESCE($5, issue_tag.category)
                RETURNING id, slug, name, description, category, created_at, updated_at
            "#,
            id,
            input.slug,
            input.name,
            input.description,
            input.category
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM issue_tag WHERE id=$1", id)
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            IssueTag,
            r#"
                SELECT id, slug, name, description, category, created_at, updated_at FROM issue_tag
            "#,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn find_by_slug(db_pool: &PgPool, slug: String) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            IssueTag,
            r#"
                SELECT id, slug, name, description, category, created_at, updated_at FROM issue_tag
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_ids(
        db_pool: &PgPool,
        ids: Vec<uuid::Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            IssueTag,
            r#"
                SELECT id, slug, name, description, category, created_at, updated_at FROM issue_tag
                WHERE id = ANY($1)
                ORDER BY name DESC
            "#,
            &ids
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &IssueTagSearch,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            IssueTag,
            r#"
                SELECT id, slug, name, description, category, created_at, updated_at FROM issue_tag
                WHERE ($1::text IS NULL OR levenshtein($1, name) <= 3)
            "#,
            search.name
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn politicians(
        db_pool: &PgPool,
        issue_tag_id: uuid::Uuid,
    ) -> Result<Vec<Politician>, sqlx::Error> {
        let records = sqlx::query_as!(
            Politician,
            r#"
                SELECT p.id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        upcoming_race_id,
                        thumbnail_image_url,
                        assets,
                        official_website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        phone,
                        party_id,
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        race_wins,
                        race_losses,
                        p.created_at,
                        p.updated_at FROM politician p
                JOIN politician_issue_tags
                ON politician_issue_tags.politician_id = p.id
                WHERE politician_issue_tags.issue_tag_id = $1
            "#,
            issue_tag_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn organizations(
        db_pool: &PgPool,
        issue_tag_id: uuid::Uuid,
    ) -> Result<Vec<Organization>, sqlx::Error> {
        let records = sqlx::query_as!(
            Organization,
            r#"
                SELECT o.id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, headquarters_phone, votesmart_sig_id, headquarters_address_id, tax_classification, o.assets, o.created_at, o.updated_at  FROM organization o
                JOIN organization_issue_tags
                ON organization_issue_tags.organization_id = o.id
                WHERE organization_issue_tags.issue_tag_id = $1
            "#, issue_tag_id).fetch_all(db_pool).await?;

        Ok(records)
    }
}
