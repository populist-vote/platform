use crate::DateTime;
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
    pub thumbnail_image_url: Option<String>,
    pub home_state: String,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
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
    pub thumbnail_image_url: Option<String>,
    pub home_state: String,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
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
    pub thumbnail_image_url: Option<String>,
    pub home_state: Option<String>,
    pub website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
}

#[derive(InputObject)]
pub struct PoliticianSearch {
    home_state: Option<String>,
    last_name: Option<String>,
}

static POLITICIAN_COLUMNS: &'static str = "id, first_name, middle_name, last_name, home_state";

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
            "INSERT INTO politician (slug, first_name, middle_name, last_name, home_state) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING *",
            slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.home_state
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
            "UPDATE politician 
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
                instagram_url = COALESCE($14, instagram_url)
            WHERE id=$1
            RETURNING *",
            id,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.nickname,
            input.preferred_name,
            input.ballot_name,
            input.description,
            input.thumbnail_image_url,
            input.home_state,
            input.website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url
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
        let records = sqlx::query_as!(Politician, "SELECT * FROM politician")
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
            "SELECT * FROM politician
             WHERE $1::text IS NULL OR home_state = $1 
             AND $2::text IS NULL OR levenshtein($2, last_name) <=5",
            search.home_state,
            search.last_name
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records.into())
    }
}

// impl PoliticianSearch {

// }
