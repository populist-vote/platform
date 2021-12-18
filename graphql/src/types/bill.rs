use crate::types::ArgumentResult;
use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::{
    models::{bill::Bill, enums::LegislationStatus},
    DateTime,
};
use sqlx::{Pool, Postgres};
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BillResult {
    id: ID,
    slug: String,
    title: String,
    vote_status: LegislationStatus,
    description: Option<String>,
    official_summary: Option<String>,
    populist_summary: Option<String>,
    full_text_url: Option<String>,
    legiscan_bill_id: Option<i32>,
    legiscan_data: serde_json::Value,
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
}

impl From<Bill> for BillResult {
    fn from(b: Bill) -> Self {
        Self {
            id: ID::from(b.id),
            slug: b.slug,
            title: b.title,
            vote_status: b.vote_status,
            description: b.description,
            official_summary: b.official_summary,
            populist_summary: b.populist_summary,
            full_text_url: b.full_text_url,
            legiscan_bill_id: b.legiscan_bill_id,
            legiscan_data: b.legiscan_data,
            history: b.history,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}
