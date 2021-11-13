use async_graphql::InputObject;
use slugify::slugify;
use sqlx::postgres::PgPool;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]

pub struct Election {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub election_date: chrono::NaiveDate,
}

#[derive(InputObject)]
pub struct CreateElectionInput {
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    // Must use format YYYY-MM-DD
    pub election_date: chrono::NaiveDate,
}

#[derive(InputObject)]
pub struct UpdateElectionInput {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub election_date: Option<chrono::NaiveDate>,
}

#[derive(InputObject)]
pub struct ElectionSearchInput {
    pub slug: Option<String>,
    pub title: Option<String>,
}

impl Election {
    pub async fn create(
        db_pool: &PgPool,
        input: &CreateElectionInput,
    ) -> Result<Self, sqlx::Error> {
        let slug = slugify!(&input.title);
        let record = sqlx::query_as!(
            Election,
            r#"INSERT INTO election
           (slug, title, description, election_date)
           VALUES ($1, $2, $3, $4)
           RETURNING id, slug, title, description, election_date"#,
            slug,
            input.title,
            input.description,
            input.election_date
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateElectionInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Election,
            r#"UPDATE election
            SET slug = COALESCE($2, slug),
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                election_date = COALESCE($5, election_date)
            WHERE id=$1    
            RETURNING id, slug, title, description, election_date"#,
            id,
            input.slug,
            input.title,
            input.description,
            input.election_date
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM election WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Election,
            r#"SELECT id, slug, title, description, election_date FROM election"#,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records.into())
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &ElectionSearchInput,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Election,
            r#"SELECT id, slug, title, description, election_date FROM election
            WHERE $1::text IS NULL OR slug = $1
            AND $2::text IS NULL OR title = $2"#,
            search.slug,
            search.title,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records.into())
    }
}
