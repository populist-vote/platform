use crate::{DateTime, IssueTag};
use async_graphql::InputObject;
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::{BallotMeasureStatus, State};

#[derive(FromRow, Debug, Clone)]
pub struct BallotMeasure {
    // required fields
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub status: BallotMeasureStatus,
    pub election_id: uuid::Uuid,
    pub state: State,
    pub ballot_measure_code: String,
    pub measure_type: String, //perhaps make enum later
    pub definitions: String,  // markdown list of bulleted items
    //optional fields
    pub yes_votes: Option<i32>,
    pub no_votes: Option<i32>,
    pub num_precincts_reporting: Option<i32>,
    pub total_precincts: Option<i32>,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct UpsertBallotMeasureInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub status: Option<BallotMeasureStatus>,
    pub state: Option<State>,
    pub ballot_measure_code: Option<String>,
    pub measure_type: Option<String>,
    pub definitions: Option<String>,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
}

#[derive(InputObject)]
pub struct BallotMeasureSearch {
    slug: Option<String>,
    title: Option<String>,
    state: Option<State>,
    status: Option<BallotMeasureStatus>,
}

impl BallotMeasure {
    pub async fn upsert(
        db_pool: &PgPool,
        election_id: uuid::Uuid,
        input: &UpsertBallotMeasureInput,
    ) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let record = sqlx::query_as!(
            BallotMeasure,
            r#"
                INSERT INTO ballot_measure 
                (id, election_id, slug, title, status, description, official_summary, 
                populist_summary, full_text_url, state, ballot_measure_code, 
                measure_type, definitions) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (id) DO UPDATE SET
                    slug = COALESCE($3, ballot_measure.slug),
                    title = COALESCE($4, ballot_measure.title),
                    status = COALESCE($5, ballot_measure.status),
                    description = COALESCE($6, ballot_measure.description),
                    official_summary = COALESCE($7, ballot_measure.official_summary),
                    populist_summary = COALESCE($8, ballot_measure.populist_summary),
                    full_text_url = COALESCE($9, ballot_measure.full_text_url),
                    state = COALESCE($10, ballot_measure.state),
                    ballot_measure_code = COALESCE($11, ballot_measure.ballot_measure_code),
                    measure_type = COALESCE($12, ballot_measure.measure_type),
                    definitions = COALESCE($13, ballot_measure.definitions)
                RETURNING id, election_id, slug, title, status AS "status: BallotMeasureStatus", description, official_summary, populist_summary, full_text_url, state AS "state:State", ballot_measure_code, measure_type, definitions, yes_votes, no_votes, num_precincts_reporting, total_precincts, created_at, updated_at
            "#,
            id,
            election_id,
            input.slug,
            input.title,
            input.status as Option<BallotMeasureStatus>,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.state as Option<State>,
            input.ballot_measure_code,
            input.measure_type,
            input.definitions
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ballot_measure WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(BallotMeasure, r#"SELECT id, election_id, slug, title, status AS "status: BallotMeasureStatus", state AS "state:State", ballot_measure_code, measure_type, definitions, description, official_summary, populist_summary, full_text_url,  yes_votes, no_votes, num_precincts_reporting, total_precincts, created_at, updated_at FROM ballot_measure"#,)
            .fetch_all(db_pool)
            .await?;
        Ok(records)
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &BallotMeasureSearch,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            BallotMeasure,
            r#"
                SELECT id, election_id, slug, title, status AS "status: BallotMeasureStatus", state AS "state:State", ballot_measure_code, measure_type, definitions, description, official_summary, populist_summary, full_text_url,  yes_votes, no_votes, num_precincts_reporting, total_precincts, created_at, updated_at FROM ballot_measure
                WHERE ($1::text IS NULL OR slug = $1)
                AND ($2::text IS NULL OR levenshtein($2, title) <=5)
                AND ($3::state IS NULL OR state = $3)
                AND ($4::ballot_measure_status IS NULL OR status = $4)
            "#,
            search.slug,
            search.title,
            search.state as Option<State>,
            search.status as Option<BallotMeasureStatus>
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN ballot_measure_issue_tags bmit
                ON bmit.issue_tag_id = it.id
                WHERE bmit.ballot_measure_id = $1
            "#,
            bill_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
