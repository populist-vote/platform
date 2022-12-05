use crate::{context::ApiContext, types::ArgumentResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::models::{bill::Bill, enums::LegislationStatus};
use legiscan::Bill as LegiscanBill;
use sqlx::{types::Json, Row};
use uuid::Uuid;

use super::{IssueTagResult, PoliticianResult};
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
}

#[ComplexObject]
impl BillResult {
    async fn arguments(&self, ctx: &Context<'_>) -> Result<Vec<ArgumentResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Bill::arguments(&db_pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(ArgumentResult::from).collect();
        Ok(results)
    }

    async fn legiscan_data(&self, ctx: &Context<'_>) -> Result<LegiscanBill> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let record = sqlx::query(
            r#"
                SELECT legiscan_data FROM bill
                WHERE id=$1
            "#,
        )
        .bind(Uuid::parse_str(&self.id).unwrap())
        .fetch_one(&db_pool)
        .await?;

        let legiscan_data: Json<LegiscanBill> = record.get(0);

        Ok(legiscan_data.0)
    }

    async fn issue_tags(&self, ctx: &Context<'_>) -> Result<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Bill::issue_tags(&db_pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(IssueTagResult::from).collect();
        Ok(results)
    }

    async fn sponsors(&self, ctx: &Context<'_>) -> Result<Vec<PoliticianResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Bill::sponsors(&db_pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(PoliticianResult::from).collect();
        Ok(results)
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
        }
    }
}
