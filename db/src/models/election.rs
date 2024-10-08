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
    pub municipality: Option<String>,
    pub election_date: chrono::NaiveDate,
}

#[derive(InputObject, Default, Debug)]
pub struct UpsertElectionInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<State>,
    pub municipality: Option<String>,
    /// Must use format YYYY-MM-DD
    pub election_date: Option<chrono::NaiveDate>,
}

#[derive(InputObject, Default, Debug)]
pub struct ElectionFilter {
    pub query: Option<String>,
    pub state: Option<State>,
    pub year: Option<i32>,
    pub municipality: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
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
                (id, slug, title, description, state, municipality, election_date)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    slug = COALESCE($2, election.slug),
                    title = COALESCE($3, election.title),
                    description = COALESCE($4, election.description),
                    state = COALESCE($5, election.state),
                    municipality = COALESCE($6, election.municipality),
                    election_date = COALESCE($7, election.election_date)
                RETURNING id, slug, title, description, state AS "state:State", municipality, election_date
            "#,
            id,
            input.slug,
            input.title,
            input.description,
            input.state as Option<State>,
            input.municipality,
            input.election_date
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn upsert_from_source(
        db_pool: &PgPool,
        input: &UpsertElectionInput,
    ) -> Result<Self, sqlx::Error> {
        input
            .slug
            .as_ref()
            .ok_or("slug is required")
            .map_err(|err| sqlx::Error::AnyDriverError(err.into()))?;

        sqlx::query_as!(
            Election,
            r#"
                INSERT INTO election
                (slug, title, description, state, municipality, election_date)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (slug) DO UPDATE SET
                    title = COALESCE($2, election.title),
                    description = COALESCE($3, election.description),
                    state = COALESCE($4, election.state),
                    municipality = COALESCE($5, election.municipality),
                    election_date = COALESCE($6, election.election_date)
                RETURNING id, slug, title, description, state AS "state:State", municipality, election_date
            "#,
            input.slug,
            input.title,
            input.description,
            input.state as Option<State>,
            input.municipality,
            input.election_date
        )
        .fetch_one(db_pool)
        .await
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
                SELECT id, slug, title, description, state AS "state:State", municipality, election_date
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
            r#"SELECT id, slug, title, description, state AS "state:State", municipality, election_date FROM election"#,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn filter(
        db_pool: &PgPool,
        filter: &ElectionFilter,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Election,
            r#"
            SELECT
                id,
                slug,
                title,
                description,
                state AS "state:State",
                municipality,
                election_date
            FROM
                election e,
                to_tsvector(title || ' ' || COALESCE(description, '') || ' ' || COALESCE(municipality, '') || ' ' || COALESCE(state::text, '')) document,
                websearch_to_tsquery($1) query,
                NULLIF(ts_rank(to_tsvector(title), websearch_to_tsquery ($1::text)), 0)
                rank
            WHERE ($3::int IS NULL OR EXTRACT(YEAR FROM e.election_date) = $3)
            AND (
                (($1::text = '') IS NOT FALSE OR query @@ document)
                AND (($2::state IS NULL OR e.state = $2)
                OR (state IS NULL AND municipality IS NULL)) -- Always return national elections
            )
            ORDER BY election_date ASC
            "#,
            filter.query,
            filter.state as Option<State>,
            filter.year,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
