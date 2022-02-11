use crate::{
    models::enums::{ArgumentPosition, AuthorType, LegislationStatus},
    Argument, CreateArgumentInput, DateTime, IssueTag,
};
use async_graphql::InputObject;
use serde_json::Value;
use slugify::slugify;
use sqlx::{postgres::PgPool, FromRow};

#[derive(FromRow, Debug, Clone)]
pub struct Bill {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub bill_number: String,
    pub legislation_status: LegislationStatus,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub votesmart_bill_id: Option<i32>,
    pub legiscan_bill_id: Option<i32>,
    pub history: Value,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct CreateBillInput {
    pub slug: Option<String>,
    pub title: String,
    pub bill_number: String,
    pub legislation_status: LegislationStatus,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub legiscan_bill_id: Option<i32>,
    pub legiscan_data: Option<Value>,
    pub votesmart_bill_id: Option<i32>,
    pub arguments: Option<Vec<CreateArgumentInput>>,
}

#[derive(InputObject, Default)]
pub struct UpdateBillInput {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub bill_number: String,
    pub legislation_status: Option<LegislationStatus>,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub legiscan_bill_id: Option<i32>,
    pub legiscan_data: Option<Value>,
    pub arguments: Option<Vec<CreateArgumentInput>>,
}

#[derive(InputObject)]
pub struct BillSearch {
    slug: Option<String>,
    title: Option<String>,
    bill_number: Option<String>,
    legislation_status: Option<LegislationStatus>,
}

impl Default for BillSearch {
    fn default() -> Self {
        Self {
            slug: None,
            title: None,
            bill_number: None,
            legislation_status: None,
        }
    }
}

impl Bill {
    pub async fn create(db_pool: &PgPool, input: &CreateBillInput) -> Result<Self, sqlx::Error> {
        let slug = match input.slug.clone() {
            None => slugify!(&input.bill_number),
            Some(slug) => slug,
        };

        let legiscan_data = input
            .legiscan_data
            .clone()
            .unwrap_or_else(|| serde_json::from_str("{}").unwrap());

        let record = sqlx::query_as!(
            Bill,
            r#"
                INSERT INTO bill (slug, title, bill_number, legislation_status, description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, votesmart_bill_id) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (slug) DO UPDATE
                SET title = $2
                RETURNING id, slug, title, bill_number, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, history, votesmart_bill_id, created_at, updated_at
            "#,
            slug,
            input.title,
            input.bill_number,
            input.legislation_status as LegislationStatus,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.legiscan_bill_id,
            legiscan_data,
            input.votesmart_bill_id,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: Option<uuid::Uuid>,
        legiscan_bill_id: Option<i32>,
        input: &UpdateBillInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Bill,
            r#"
                UPDATE bill
                SET slug = COALESCE($3, slug),
                    title = COALESCE($4, title),
                    bill_number = COALESCE($5,bill_number),
                    legislation_status = COALESCE($6, legislation_status),
                    description = COALESCE($7, description),
                    official_summary = COALESCE($8, official_summary),
                    populist_summary = COALESCE($9, populist_summary),
                    full_text_url = COALESCE($10, full_text_url),
                    legiscan_bill_id = COALESCE($11, legiscan_bill_id), 
                    legiscan_data = COALESCE($12, legiscan_data)
                WHERE id=$1
                OR legiscan_bill_id=$2
                RETURNING id, slug, title, bill_number, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, history, votesmart_bill_id, created_at, updated_at
            "#,
            id,
            legiscan_bill_id,
            input.slug,
            input.title,
            input.bill_number,
            input.legislation_status as Option<LegislationStatus>,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.legiscan_bill_id,
            input.legiscan_data
        ).fetch_one(db_pool).await?;
        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM bill WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    // this table is too big to run this query, its too expensive and will blow up heroku
    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(Bill, r#"
            SELECT id, slug, title, bill_number, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, history, votesmart_bill_id, created_at, updated_at FROM bill"#)
            .fetch_all(db_pool)
            .await?;
        Ok(records)
    }

    pub async fn search(db_pool: &PgPool, search: &BillSearch) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Bill,
            r#"
                SELECT id, slug, title, bill_number, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, history, votesmart_bill_id, created_at, updated_at FROM bill
                WHERE ($1::text IS NULL OR slug = $1)
                AND ($2::text IS NULL OR title ILIKE $2)
                AND ($3::legislation_status IS NULL OR legislation_status = $3)
                AND ($4::text IS NULL OR bill_number ILIKE $4)
            "#,
            search.slug,
            search.title,
            search.legislation_status as Option<LegislationStatus>,
            search.bill_number
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn find_by_id(_db_pool: &PgPool) -> Result<Self, sqlx::Error> {
        todo!()
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN bill_issue_tags
                ON bill_issue_tags.issue_tag_id = it.id
                WHERE bill_issue_tags.bill_id = $1
            "#,
            bill_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn create_bill_argument(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
        author_id: uuid::Uuid,
        input: &CreateArgumentInput,
    ) -> Result<Argument, sqlx::Error> {
        let record = sqlx::query_as_unchecked!(
            Argument,
            r#"
                WITH ins_argument AS (
                    INSERT INTO argument (author_id, title, position, body) 
                    VALUES ($2, $3, $4, $5) 
                    RETURNING id, author_id, title, position, body, created_at, updated_at
                ),
                ins_bill_argument AS (
                    INSERT INTO bill_arguments (bill_id, argument_id) 
                    VALUES ($1, (SELECT id FROM ins_argument))
                )
                SELECT ins_argument.id, ins_argument.author_id, a.author_type AS "author_type:AuthorType", ins_argument.title, ins_argument.position AS "position:ArgumentPosition", ins_argument.body, ins_argument.created_at, ins_argument.updated_at
                FROM ins_argument JOIN author AS a ON a.id = ins_argument.author_id
            "#,
            bill_id,
            author_id,
            input.title,
            input.position as ArgumentPosition,
            input.body,
        ).fetch_one(db_pool).await?;

        Ok(record)
    }

    pub async fn arguments(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
    ) -> Result<Vec<Argument>, sqlx::Error> {
        let records = sqlx::query_as!(Argument,
            r#"
                SELECT arg.id, arg.author_id, author.author_type AS "author_type:AuthorType", title, position AS "position:ArgumentPosition", body, arg.created_at, arg.updated_at 
                FROM argument AS arg
                JOIN author ON author.id = arg.author_id
                JOIN bill_arguments ON bill_arguments.argument_id = arg.id
                WHERE bill_arguments.bill_id = $1
            "#,
            bill_id
        ).fetch_all(db_pool).await?;

        Ok(records)
    }

    pub async fn connect_issue_tag(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
        issue_tag_id: uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Bill,
            r#"
                INSERT INTO bill_issue_tags (bill_id, issue_tag_id) 
                VALUES ($1, $2)
            "#,
            bill_id,
            issue_tag_id
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }
}

// impl Default for Bill {
//     fn default() -> Bill {
//         Bill {
//             id: uuid::Uuid::new_v4(),
//             slug: "some-piece-of-legislation".to_string(),
//             title: "Some Piece of Legislation".to_string(),
//             legislation_status: LegislationStatus::UNDECIDED,
//             description: None,
//             official_summary: None,
//             populist_summary: None,
//             full_text_url: None,
//         }
//     }
// }
