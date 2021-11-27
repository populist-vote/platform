use crate::{
    models::enums::{ArgumentPosition, AuthorType, LegislationStatus},
    Argument, CreateArgumentInput, DateTime, IssueTag,
};
use async_graphql::InputObject;
use serde_json::Value;
use slugify::slugify;
use sqlx::FromRow;
use sqlx::{postgres::PgPool};

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
    pub legiscan_bill_id: Option<i32>,
    pub legiscan_data: Option<Value>,
    pub arguments: Option<Vec<CreateArgumentInput>>,
}

#[derive(InputObject, Default)]
pub struct UpdateBillInput {
    pub slug: Option<String>,
    pub name: Option<String>,
    pub vote_status: Option<LegislationStatus>,
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
    name: Option<String>,
    vote_status: Option<LegislationStatus>,
}

impl Bill {
    pub async fn create(db_pool: &PgPool, input: &CreateBillInput) -> Result<Self, sqlx::Error> {
        let slug = slugify!(&input.name); // TODO run a query and ensure this is Unique

        let legiscan_data = input
            .legiscan_data
            .clone()
            .unwrap_or(serde_json::from_str("{}").unwrap());

        let record = sqlx::query_as!(
            Bill,
            r#"
                INSERT INTO bill (slug, name, vote_status, description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
                RETURNING id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, created_at, updated_at
            "#,
            slug,
            input.name,
            input.vote_status as LegislationStatus,
            input.description,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.legiscan_bill_id,
            legiscan_data
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
                    name = COALESCE($4, name),
                    vote_status = COALESCE($5, vote_status),
                    description = COALESCE($6, description),
                    official_summary = COALESCE($7, official_summary),
                    populist_summary = COALESCE($8, populist_summary),
                    full_text_url = COALESCE($9, full_text_url),
                    legiscan_bill_id = COALESCE($10, legiscan_bill_id), 
                    legiscan_data = COALESCE($11, legiscan_data)
                WHERE id=$1
                OR legiscan_bill_id=$2
                RETURNING id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, created_at, updated_at
            "#,
            id,
            legiscan_bill_id,
            input.slug,
            input.name,
            input.vote_status as Option<LegislationStatus>,
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

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(Bill, r#"SELECT id, slug, name, vote_status AS "vote_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_data, created_at, updated_at FROM bill"#,)
            .fetch_all(db_pool)
            .await?;
        Ok(records)
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
                SELECT it.id, slug, name, description, it.created_at, it.updated_at FROM issue_tag it
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
//             name: "Some Piece of Legislation".to_string(),
//             vote_status: LegislationStatus::UNDECIDED,
//             description: None,
//             official_summary: None,
//             populist_summary: None,
//             full_text_url: None,
//         }
//     }
// }
