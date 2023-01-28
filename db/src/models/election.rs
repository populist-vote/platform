use async_graphql::InputObject;
use sqlx::postgres::PgPool;
use sqlx::FromRow;

use super::enums::State;

#[derive(FromRow, Debug, Clone)]
pub struct Election {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub state: Option<State>,
    pub election_date: chrono::NaiveDate,
}

#[derive(InputObject)]
pub struct UpsertElectionInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<State>,
    /// Must use format YYYY-MM-DD
    pub election_date: Option<chrono::NaiveDate>,
}

#[derive(InputObject, Default)]
pub struct ElectionSearchInput {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub state: Option<State>,
}

impl Election {
    pub async fn upsert(
        db_pool: &PgPool,
        input: &UpsertElectionInput,
    ) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(uuid::Uuid::new_v4);
        let record = sqlx::query_as!(
            Election,
            r#"
                INSERT INTO election
                (id, slug, title, description, state, election_date)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO UPDATE SET
                    slug = COALESCE($2, election.slug),
                    title = COALESCE($3, election.title),
                    description = COALESCE($4, election.description),
                    state = COALESCE($5, election.state),
                    election_date = COALESCE($6, election.election_date)
                RETURNING id, slug, title, description, state AS "state:State", election_date
            "#,
            id,
            input.slug,
            input.title,
            input.description,
            input.state as Option<State>,
            input.election_date
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM election WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Election,
            r#"
                SELECT id, slug, title, description, state AS "state:State", election_date
                FROM election
                WHERE id=$1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Election,
            r#"SELECT id, slug, title, description, state AS "state:State", election_date FROM election"#,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &ElectionSearchInput,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Election,
            r#"
                SELECT id, slug, title, description, state AS "state:State", election_date FROM election
                WHERE ($1::text IS NULL OR slug = $1)
                AND ($2::text IS NULL OR title = $2)
                AND ($3::state IS NULL OR state = $3)
                ORDER BY election_date ASC
            "#,
            search.slug,
            search.title,
            search.state as Option<State>,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
