use crate::{DateTime, IssueTag};
use async_graphql::InputObject;
use slugify::slugify;
use sqlx::postgres::PgPool;
use sqlx::FromRow;

use super::enums::{LegislationStatus, State};

#[derive(FromRow, Debug, Clone)]
pub struct BallotMeasure {
    // required fields
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub legislation_status: LegislationStatus,
    pub election_id: uuid::Uuid,
    pub ballot_state: State,
    pub ballot_measure_code: String,
    pub measure_type: String, //perhaps make enum later
    pub definitions: String,  // markdown list of bulleted items

    //optional fields
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,

    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct CreateBallotMeasureInput {
    pub slug: Option<String>,
    pub title: String,
    pub legislation_status: LegislationStatus,
    pub ballot_state: State,
    pub ballot_measure_code: String,
    pub measure_type: String,
    pub definitions: String,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateBallotMeasureInput {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub legislation_status: Option<LegislationStatus>,
    pub ballot_state: Option<State>,
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
    ballot_state: Option<State>,
    legislation_status: Option<LegislationStatus>,
}

impl BallotMeasure {
    pub async fn create(
        db_pool: &PgPool,
        election_id: uuid::Uuid,
        input: &CreateBallotMeasureInput,
    ) -> Result<Self, sqlx::Error> {
        let slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&input.title),
        };
        let record = sqlx::query_as!(
            BallotMeasure,
            r#"
                INSERT INTO ballot_measure 
                (election_id, slug, title, legislation_status, description, official_summary, 
                populist_summary, full_text_url, ballot_state, ballot_measure_code, 
                measure_type, definitions) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) 
                RETURNING id, election_id, slug, title, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, ballot_state AS "ballot_state:State", ballot_measure_code, measure_type, definitions, created_at, updated_at
            "#,
            election_id,
            slug,
            input.title,
            input.legislation_status as LegislationStatus,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.ballot_state as State,
            input.ballot_measure_code,
            input.measure_type,
            input.definitions
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateBallotMeasureInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            BallotMeasure,
            r#"
                UPDATE ballot_measure
                SET slug = COALESCE($2, slug),
                    title = COALESCE($3, title),
                    legislation_status = COALESCE($4, legislation_status),
                    ballot_state = COALESCE($5, ballot_state),
                    ballot_measure_code = COALESCE($6, ballot_measure_code),
                    measure_type = COALESCE($7, measure_type),
                    definitions = COALESCE($8, definitions),
                    description = COALESCE($9, description),
                    official_summary = COALESCE($10, official_summary),
                    populist_summary = COALESCE($11, populist_summary),
                    full_text_url = COALESCE($12, full_text_url)
                WHERE id=$1    
                RETURNING id, election_id, slug, title, legislation_status AS "legislation_status:LegislationStatus", ballot_state AS "ballot_state:State", ballot_measure_code, measure_type, definitions, description, official_summary, populist_summary, full_text_url, created_at, updated_at
            "#,
            id,
            input.slug,
            input.title,
            input.legislation_status as Option<LegislationStatus>,
            input.ballot_state as Option<State>,
            input.ballot_measure_code,
            input.measure_type,
            input.definitions,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url
        ).fetch_one(db_pool).await?;
        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ballot_measure WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(BallotMeasure, r#"SELECT id, election_id, slug, title, legislation_status AS "legislation_status:LegislationStatus", ballot_state AS "ballot_state:State", ballot_measure_code, measure_type, definitions, description, official_summary, populist_summary, full_text_url, created_at, updated_at FROM ballot_measure"#,)
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
                SELECT id, election_id, slug, title, legislation_status AS "legislation_status:LegislationStatus", ballot_state AS "ballot_state:State", ballot_measure_code, measure_type, definitions, description, official_summary, populist_summary, full_text_url, created_at, updated_at FROM ballot_measure
                WHERE ($1::text IS NULL OR slug = $1)
                AND ($2::text IS NULL OR levenshtein($2, title) <=5)
                AND ($3::state IS NULL OR ballot_state = $3)
                AND ($4::legislation_status IS NULL OR legislation_status = $4)
            "#,
            search.slug,
            search.title,
            search.ballot_state as Option<State>,
            search.legislation_status as Option<LegislationStatus>
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
