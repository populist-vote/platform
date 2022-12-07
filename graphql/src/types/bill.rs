use crate::{context::ApiContext, types::ArgumentResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{
        bill::Bill,
        enums::{BillType, LegislationStatus, PoliticalScope, State},
    },
    Chamber, PublicVotes,
};
use legiscan::Bill as LegiscanBill;
use sqlx::{types::Json, Row};
use std::str::FromStr;
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
    votesmart_bill_id: Option<i32>,
    legiscan_bill_id: Option<i32>,
    legiscan_committee_name: Option<String>,
    history: serde_json::Value,
    state: Option<State>,
    chamber: Option<Chamber>,
    bill_type: BillType,
    political_scope: PoliticalScope,
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

    async fn public_votes(&self, ctx: &Context<'_>) -> Result<PublicVotes> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let results = sqlx::query_as!(
            PublicVotes,
            r#"
                SELECT SUM(CASE WHEN position = 'support' THEN 1 ELSE 0 END) as support,
                       SUM(CASE WHEN position = 'neutral' THEN 1 ELSE 0 END) as neutral,
                       SUM(CASE WHEN position = 'oppose' THEN 1 ELSE 0 END) as oppose
                FROM bill_public_votes WHERE bill_id = $1
            "#,
            Uuid::parse_str(&self.id).unwrap(),
        )
        .fetch_one(&db_pool)
        .await?;

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
            votesmart_bill_id: b.votesmart_bill_id,
            legiscan_bill_id: b.legiscan_bill_id,
            legiscan_committee_name: b.legiscan_committee,
            history: b.history,
            state: b.state,
            chamber: b.chamber,
            bill_type: BillType::from_str(&b.bill_type).unwrap_or_default(),
            political_scope: b.political_scope,
        }
    }
}
