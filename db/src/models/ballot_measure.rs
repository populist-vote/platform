use crate::{DateTime, State};
use async_graphql::InputObject;
use slugify::slugify;
use sqlx::postgres::PgPool;
use sqlx::FromRow;

use super::legislation::LegislationStatus;

#[derive(FromRow, Debug, Clone)]
pub struct BallotMeasure {
    // required fields
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
    pub vote_status: LegislationStatus,
    pub election_id: uuid::Uuid,
    pub state: State,
    pub ballot_measure_code: String,
    pub measure_type: String, //perhaps make enum later
    pub definitions: String, // makrdown list of bulleted items

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
    pub name: String,
    pub vote_status: LegislationStatus,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateBallotMeasureInput {
    pub slug: Option<String>,
    pub name: Option<String>,
    pub vote_status: Option<LegislationStatus>,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
}

#[derive(InputObject)]
pub struct BallotMeasureSearch {
    slug: Option<String>,
    name: Option<String>,
    vote_status: Option<LegislationStatus>,
}

impl BallotMeasure {
    pub async fn create(db_pool: &PgPool, election_id: uuid::Uuid, input: &CreateBallotMeasureInput) -> Result<Self, sqlx::Error> {
        let slug = slugify!(&input.name); // TODO run a query and ensure this is Unique
        let record = sqlx::query_as!(
            BallotMeasure,
            r#"INSERT INTO ballot_measure (election_id, slug, name, vote_status, description, official_summary, populist_summary, full_text_url, state, ballot_measure_code, measure_type, definitions) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) 
            RETURNING id, election_id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, state, ballot_measure_code, measure_type, definitions, created_at, updated_at"#,
            election_id,
            slug,
            input.name,
            input.vote_status as LegislationStatus,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.state,
            input.ballot_measure_code, 
            input.measure_type, 
            input.definitions
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateBallotMeasureInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            BallotMeasure,
            r#"UPDATE ballot_measure
            SET slug = COALESCE($2, slug),
                name = COALESCE($3, name),
                vote_status = COALESCE($4, vote_status),
                description = COALESCE($5, description),
                official_summary = COALESCE($6, official_summary),
                populist_summary = COALESCE($7, populist_summary),
                full_text_url = COALESCE($8, full_text_url)
            WHERE id=$1    
            RETURNING id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, created_at, updated_at"#,
            id,
            input.slug,
            input.name,
            input.vote_status as Option<LegislationStatus>,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url
        ).fetch_one(db_pool).await?;
        Ok(record.into())
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ballot_measure WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(BallotMeasure, r#"SELECT id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, created_at, updated_at FROM ballot_measure"#,)
            .fetch_all(db_pool)
            .await?;
        Ok(records.into())
    }

    pub async fn search(db_pool: &PgPool, search: &BallotMeasureSearch) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            BallotMeasure,
            r#"SELECT id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, created_at, updated_at FROM ballot_measure
             WHERE $1::text IS NULL OR slug = $1
             AND $2::text IS NULL OR levenshtein($2, name) <=5
             AND $3::vote_status IS NULL OR vote_status = $3"#,
            search.slug,
            search.name,
            search.vote_status as Option<LegislationStatus>
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records.into())
    }
}

// impl Default for BallotMeasure {
//     fn default() -> BallotMeasure {
//         BallotMeasure {
//             id: uuid::Uuid::new_v4(),
//             slug: "some-piece-of-legislation".to_string(),
//             name: "Some Piece of Legislation".to_string(),
//             vote_status: LegislationStatus::UNDECIDED,
//             description: None,
//             official_summary: None,
//             populist_summary: None,
//             full_text_url: None,
//         }
//     }
// }
