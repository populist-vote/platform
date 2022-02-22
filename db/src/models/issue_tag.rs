use crate::{
    models::enums::{PoliticalParty, State},
    DateTime, Organization, Politician,
};
use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use slugify::slugify;
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
pub struct CreateIssueTagInput {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateIssueTagInput {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(InputObject, Debug, Serialize, Deserialize)]
pub struct CreateOrConnectIssueTagInput {
    pub create: Option<Vec<CreateIssueTagInput>>,
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
    pub async fn create(
        db_pool: &PgPool,
        input: &CreateIssueTagInput,
    ) -> Result<Self, sqlx::Error> {
        let id = uuid::Uuid::new_v4();
        let slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.name),
        };
        let record = sqlx::query_as!(
            IssueTag,
            r#"
                INSERT INTO issue_tag (id, slug, name, description, category) VALUES ($1, $2, $3, $4, $5)
                RETURNING id, slug, name, description, category, created_at, updated_at
            "#,
            id,
            slug,
            input.name,
            input.description,
            input.category
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateIssueTagInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            IssueTag,
            r#"
                UPDATE issue_tag
                SET slug = COALESCE($2, slug),
                    name = COALESCE($3, name),
                    description = COALESCE($4, description),
                    category = COALESCE($5, category)
                WHERE id = $1
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
                SELECT p.id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", office_id, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, votesmart_candidate_ratings, legiscan_people_id, upcoming_race_id, p.created_at, p.updated_at FROM politician p
                JOIN politician_issue_tags
                ON politician_issue_tags.politician_id = p.id
                WHERE politician_issue_tags.issue_tag_id = $1
            "#, issue_tag_id).fetch_all(db_pool).await?;

        Ok(records)
    }

    pub async fn organizations(
        db_pool: &PgPool,
        issue_tag_id: uuid::Uuid,
    ) -> Result<Vec<Organization>, sqlx::Error> {
        let records = sqlx::query_as!(
            Organization,
            r#"
                SELECT o.id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, headquarters_phone, tax_classification, o.created_at, o.updated_at  FROM organization o
                JOIN organization_issue_tags
                ON organization_issue_tags.organization_id = o.id
                WHERE organization_issue_tags.issue_tag_id = $1
            "#, issue_tag_id).fetch_all(db_pool).await?;

        Ok(records)
    }
}
