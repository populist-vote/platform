use async_graphql::{Context, Object, ID};
use db::{
    models::{committee::Committee, enums::State},
    Bill, BillFilter, BillSort, IssueTag,
};

use crate::{
    context::ApiContext,
    relay,
    types::{BillResult, CommitteeResult, IssueTagResult},
};

#[derive(Default)]
pub struct BillQuery;

#[allow(clippy::too_many_arguments)]
#[Object]
impl BillQuery {
    async fn bills(
        &self,
        ctx: &Context<'_>,
        filter: Option<BillFilter>,
        sort: Option<BillSort>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<BillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Bill::filter(
            &db_pool,
            &filter.unwrap_or_default(),
            &sort.unwrap_or_default(),
        )
        .await?;

        println!("records.len() = {:?}", records.len());

        relay::query(
            records.into_iter().map(BillResult::from),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    async fn bill_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<BillResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let record = Bill::find_by_id(&db_pool, uuid::Uuid::parse_str(&id).unwrap())
            .await
            .unwrap();
        Some(BillResult::from(record))
    }

    async fn bills_by_ids(&self, ctx: &Context<'_>, ids: Vec<ID>) -> Vec<BillResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let records = Bill::find_by_ids(
            &db_pool,
            ids.iter()
                .map(|id| uuid::Uuid::parse_str(id).unwrap())
                .collect(),
        )
        .await
        .unwrap();
        records.into_iter().map(BillResult::from).collect()
    }

    async fn bill_by_slug(&self, ctx: &Context<'_>, slug: String) -> Option<BillResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let record = Bill::find_by_slug(&db_pool, &slug).await.unwrap();
        Some(BillResult::from(record))
    }

    /// Returns all issue tags that have an associated bill
    async fn bill_issue_tags(&self, ctx: &Context<'_>) -> Vec<IssueTagResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        sqlx::query_as!(
            IssueTag,
            r#"
            SELECT DISTINCT it.id, name, slug, description, category, it.created_at, it.updated_at FROM issue_tag it
            JOIN bill_issue_tags bit ON bit.issue_tag_id = it.id
        "#
        )
        .fetch_all(&db_pool)
        .await
        .unwrap().into_iter().map(IssueTagResult::from).collect()
    }

    /// Returns all session years that have an associated bill
    async fn bill_years(&self, ctx: &Context<'_>) -> Vec<i32> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        sqlx::query!(
            r#"
            SELECT DISTINCT
                EXTRACT(YEAR FROM start_date)::INTEGER AS session_year
            FROM
                bill
            JOIN session ON bill.session_id = session.id 
        "#
        )
        .fetch_all(&db_pool)
        .await
        .unwrap()
        .into_iter()
        .flat_map(|row| row.session_year)
        .collect()
    }

    /// Returns all committees that have an associated bill
    async fn bill_committees(&self, ctx: &Context<'_>) -> Vec<CommitteeResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        sqlx::query_as!(
            Committee,
            r#"
            SELECT DISTINCT c.id, c.slug, name, c.description, c.state AS "state: State", chair_id, c.legiscan_committee_id, c.created_at, c.updated_at FROM committee c
            JOIN bill b ON b.committee_id = c.id
        "#
        )
        .fetch_all(&db_pool)
        .await
        .unwrap().into_iter().map(CommitteeResult::from).collect()
    }
}
