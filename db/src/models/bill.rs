use crate::DateTime;
use async_graphql::InputObject;
use serde_json::Value;
use slugify::slugify;
use sqlx::postgres::PgPool;
use sqlx::FromRow;

use super::legislation::LegislationStatus;

#[derive(FromRow, Debug, Clone)]
pub struct Bill {
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
    pub vote_status: LegislationStatus,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub legiscan_bill_id: Option<i32>,
    pub legiscan_data: Value,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct CreateBillInput {
    pub slug: Option<String>,
    pub name: String,
    pub vote_status: LegislationStatus,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateBillInput {
    pub slug: Option<String>,
    pub name: Option<String>,
    pub vote_status: Option<LegislationStatus>,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
}

#[derive(InputObject)]
pub struct BillSearch {
    slug: Option<String>,
    name: Option<String>,
    vote_status: Option<LegislationStatus>,
}

impl Bill {
    pub async fn create(db_pool: &PgPool, input: &CreateBillInput) -> Result<Self, sqlx::Error> {
        let slug = slugify!(&input.name); // TODO run a query and ensure this is Unique
        let record = sqlx::query_as!(
            Bill,
            r#"INSERT INTO bill (slug, name, vote_status, description, official_summary, populist_summary, full_text_url) 
            VALUES ($1, $2, $3, $4, $5, $6, $7) 
            RETURNING id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, created_at, updated_at"#,
            slug,
            input.name,
            input.vote_status as LegislationStatus,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.into())
    }

    pub async fn update(
        db_pool: &PgPool,
        id: uuid::Uuid,
        input: &UpdateBillInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Bill,
            r#"UPDATE bill
            SET slug = COALESCE($2, slug),
                name = COALESCE($3, name),
                vote_status = COALESCE($4, vote_status),
                description = COALESCE($5, description),
                official_summary = COALESCE($6, official_summary),
                populist_summary = COALESCE($7, populist_summary),
                full_text_url = COALESCE($8, full_text_url)
            WHERE id=$1    
            RETURNING id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, created_at, updated_at"#,
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
        sqlx::query!("DELETE FROM bill WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(Bill, r#"SELECT id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, created_at, updated_at FROM bill"#,)
            .fetch_all(db_pool)
            .await?;
        Ok(records.into())
    }

    pub async fn search(db_pool: &PgPool, search: &BillSearch) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Bill,
            r#"SELECT id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, created_at, updated_at FROM bill
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

// impl Default for Bill {
//     fn default() -> Bill {
//         Bill {
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
