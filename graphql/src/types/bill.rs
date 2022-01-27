use crate::types::ArgumentResult;
use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::{
    models::{bill::Bill, enums::LegislationStatus},
    DateTime,
};
use legiscan::Bill as LegiscanBill;
use sqlx::{types::Json, Pool, Postgres, Row};
use uuid::Uuid;
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BillResult {
    id: ID,
    slug: String,
    title: String,
    bill_number: String,
    legislation_status: LegislationStatus,
    description: Option<String>,
    official_summary: Option<String>,
    populist_summary: Option<String>,
    full_text_url: Option<String>,
    legiscan_bill_id: Option<i32>,
    history: serde_json::Value,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl BillResult {
    async fn arguments(&self, ctx: &Context<'_>) -> FieldResult<Vec<ArgumentResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Bill::arguments(pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(ArgumentResult::from).collect();
        Ok(results)
    }

    async fn legiscan_data(&self, ctx: &Context<'_>) -> FieldResult<LegiscanBill> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();

        let record = sqlx::query(
            r#"
                SELECT legiscan_data FROM bill
                WHERE id=$1
            "#,
        )
        .bind(Uuid::parse_str(&self.id).unwrap())
        .fetch_one(pool)
        .await?;

        let legiscan_data: Json<LegiscanBill> = record.get(0);

        Ok(legiscan_data.0)
    }
}

impl From<Bill> for BillResult {
    fn from(b: Bill) -> Self {
        Self {
            id: ID::from(b.id),
            slug: b.slug,
            title: b.title,
            bill_number: b.bill_number,
            legislation_status: b.legislation_status,
            description: b.description,
            official_summary: b.official_summary,
            populist_summary: b.populist_summary,
            full_text_url: b.full_text_url,
            legiscan_bill_id: b.legiscan_bill_id,
            history: b.history,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}
