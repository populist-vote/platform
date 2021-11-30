use crate::{
    models::enums::{PoliticalParty, State},
    CreateOrConnectIssueTagInput, DateTime, IssueTag, Organization,
};
use async_graphql::InputObject;
use serde_json::Value;
use slugify::slugify;
use sqlx::{postgres::PgPool, FromRow};

#[derive(FromRow, Debug, Clone)]
pub struct Politician {
    pub id: uuid::Uuid,
    pub slug: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub preferred_name: Option<String>,
    pub ballot_name: Option<String>,
    pub description: Option<String>,
    pub home_state: State,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub office_party: Option<PoliticalParty>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Value,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject, Default)]
pub struct CreatePoliticianInput {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub slug: String,
    pub nickname: Option<String>,
    pub preferred_name: Option<String>,
    pub ballot_name: Option<String>,
    pub description: Option<String>,
    pub home_state: State,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub office_party: Option<PoliticalParty>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<Value>,
}

#[derive(InputObject, Default)]
pub struct UpdatePoliticianInput {
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub slug: Option<String>,
    pub nickname: Option<String>,
    pub preferred_name: Option<String>,
    pub ballot_name: Option<String>,
    pub description: Option<String>,
    pub home_state: Option<State>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub office_party: Option<PoliticalParty>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<Value>,
}

#[derive(InputObject)]
pub struct PoliticianSearch {
    home_state: Option<State>,
    last_name: Option<String>,
    office_party: Option<PoliticalParty>,
}

impl CreatePoliticianInput {
    fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle_name) => format!(
                "{} {} {}",
                &self.first_name,
                middle_name.to_string(),
                &self.last_name
            ),
            None => format!("{} {}", &self.first_name, &self.last_name),
        }
    }
}

impl Politician {
    pub async fn create(
        db_pool: &PgPool,
        input: &CreatePoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        let slug = slugify!(&CreatePoliticianInput::full_name(input));

        let record = sqlx::query_as!(
            Politician,
            r#"
                WITH ins_author AS (
                    INSERT INTO author (author_type) VALUES ('politician')
                    ON CONFLICT DO NOTHING
                    RETURNING id AS author_id
                ),
                p AS (
                    INSERT INTO politician (id, slug, first_name, middle_name, last_name, home_state, office_party, votesmart_candidate_id, votesmart_candidate_bio) 
                    VALUES ((SELECT author_id FROM ins_author), $1, $2, $3, $4, $5, $6, $7, $8)
                    RETURNING id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, created_at, updated_at
                )
                SELECT p.* FROM p
            "#,
            slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.home_state as State,
            input.office_party as Option<PoliticalParty>,
            input.votesmart_candidate_id,
            input.votesmart_candidate_bio
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: Option<uuid::Uuid>,
        votesmart_candidate_id: Option<i32>,
        input: &UpdatePoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
                UPDATE politician 
                SET first_name = COALESCE($3, first_name),
                    middle_name = COALESCE($4, middle_name),
                    last_name = COALESCE($5, last_name),
                    nickname = COALESCE($6, nickname),
                    preferred_name = COALESCE($7, preferred_name),
                    ballot_name = COALESCE($8, ballot_name),
                    description = COALESCE($9, description),
                    thumbnail_image_url = COALESCE($10, thumbnail_image_url),
                    home_state= COALESCE($11, home_state),
                    website_url = COALESCE($12, website_url),
                    facebook_url = COALESCE($13, facebook_url),
                    twitter_url = COALESCE($14, twitter_url),
                    instagram_url = COALESCE($15, instagram_url),
                    office_party = COALESCE($16, office_party),
                    votesmart_candidate_bio = COALESCE($17, votesmart_candidate_bio)
                WHERE id=$1
                OR votesmart_candidate_id = $2
                RETURNING id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, created_at, updated_at
            "#,
            id,
            votesmart_candidate_id,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.nickname,
            input.preferred_name,
            input.ballot_name,
            input.description,
            input.thumbnail_image_url,
            input.home_state as Option<State>,
            input.website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.office_party as Option<PoliticalParty>,
            input.votesmart_candidate_bio
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM politician WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(Politician, r#"SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, created_at, updated_at FROM politician"#,)
            .fetch_all(db_pool)
            .await?;
        Ok(records)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(Politician,
            r#"
                SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, created_at, updated_at FROM politician
                WHERE id = $1
            "#, id)
            .fetch_one(db_pool).await?;
        Ok(record)
    }

    pub async fn find_by_slug(db_pool: &PgPool, slug: String) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(Politician,
            r#"
                SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, created_at, updated_at FROM politician
                WHERE slug = $1
            "#, slug)
            .fetch_one(db_pool).await?;

        Ok(record)
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &PoliticianSearch,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Politician,
            r#"SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, created_at, updated_at FROM politician
             WHERE $1::state IS NULL OR home_state = $1
             AND $2::text IS NULL OR levenshtein($2, last_name) <=5
             AND $3::political_party IS NULL OR office_party = $3"#,
            search.home_state as Option<State>,
            search.last_name,
            search.office_party as Option<PoliticalParty>,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn endorsements(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<Organization>, sqlx::Error> {
        let records = sqlx::query_as!(Organization,
            r#"
                SELECT o.id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, headquarters_phone, tax_classification, o.created_at, o.updated_at  FROM organization o
                JOIN politician_endorsements
                ON politician_endorsements.organization_id = o.id
                WHERE politician_endorsements.politician_id = $1
            "#, 
        politician_id).fetch_all(db_pool).await?;

        Ok(records)
    }

    pub async fn connect_issue_tag(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
        issue_tag_id: uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Politician,
            r#"
                INSERT INTO politician_issue_tags (politician_id, issue_tag_id) 
                VALUES ($1, $2)
            "#,
            politician_id,
            issue_tag_id
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, it.created_at, it.updated_at FROM issue_tag it
                JOIN politician_issue_tags
                ON politician_issue_tags.issue_tag_id = it.id
                WHERE politician_issue_tags.politician_id = $1
            "#,
            politician_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
