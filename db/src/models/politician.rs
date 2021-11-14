use super::enums::{PoliticalParty, State};
use crate::{CreateOrConnectIssueTagInput, DateTime, IssueTag, Organization};
use async_graphql::InputObject;
use slugify::slugify;
use sqlx::postgres::PgPool;
use sqlx::FromRow;
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
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct CreatePoliticianInput {
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
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
}

#[derive(InputObject)]
pub struct UpdatePoliticianInput {
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
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
        let slug = slugify!(&CreatePoliticianInput::full_name(&input)); // TODO run a query and ensure this is Unique
        let record = sqlx::query_as!(
            Politician,
            r#"INSERT INTO politician (slug, first_name, middle_name, last_name, home_state, office_party) 
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", created_at, updated_at"#,
            slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.home_state as State,
            input.office_party as Option<PoliticalParty>,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdatePoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"UPDATE politician 
            SET first_name = COALESCE($2, first_name),
                middle_name = COALESCE($3, middle_name),
                last_name = COALESCE($4, last_name),
                nickname = COALESCE($5, nickname),
                preferred_name = COALESCE($6, preferred_name),
                ballot_name = COALESCE($7, ballot_name),
                description = COALESCE($8, description),
                thumbnail_image_url = COALESCE($9, thumbnail_image_url),
                home_state= COALESCE($10, home_state),
                website_url = COALESCE($11, website_url),
                facebook_url = COALESCE($12, facebook_url),
                twitter_url = COALESCE($13, twitter_url),
                instagram_url = COALESCE($14, instagram_url),
                office_party = COALESCE($15, office_party)
            WHERE id=$1
            RETURNING id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", created_at, updated_at"#,
            id,
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
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM politician WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(Politician, r#"SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", created_at, updated_at FROM politician"#,)
            .fetch_all(db_pool)
            .await?;
        Ok(records.into())
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &PoliticianSearch,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Politician,
            r#"SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", created_at, updated_at FROM politician
             WHERE $1::state IS NULL OR home_state = $1
             AND $2::text IS NULL OR levenshtein($2, last_name) <=5
             AND $3::political_party IS NULL OR office_party = $3"#,
            search.home_state as Option<State>,
            search.last_name,
            search.office_party as Option<PoliticalParty>,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records.into())
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

        Ok(records.into())
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

        Ok(records.into())
    }
}

// impl PoliticianSearch {

// }
