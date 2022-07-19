use async_graphql::{Context, FieldResult, Object, Result, ID};
use db::{Election, ElectionSearchInput};

use crate::{context::ApiContext, types::ElectionResult};

#[derive(Default)]
pub struct ElectionQuery;

#[Object]
impl ElectionQuery {
    async fn elections(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by slug or title")] search: Option<ElectionSearchInput>,
    ) -> FieldResult<Vec<ElectionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Election::search(&db_pool, &search.unwrap_or_default()).await?;
        let results = records.into_iter().map(ElectionResult::from).collect();
        Ok(results)
    }

    async fn next_election(&self, ctx: &Context<'_>) -> FieldResult<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            Election,
            r#"SELECT
                id,
                slug,
                title,
                description,
                election_date
            FROM
                election
            WHERE election_date > NOW()
            ORDER BY
                election_date ASC
            LIMIT 1"#
        )
        .fetch_one(&db_pool)
        .await?;
        Ok(record.into())
    }

    async fn election_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            Election,
            "SELECT id, slug, title, description, election_date FROM election WHERE id = $1",
            uuid::Uuid::parse_str(id.as_str()).unwrap()
        )
        .fetch_one(&db_pool)
        .await?;
        Ok(record.into())
    }

    async fn election_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            Election,
            "SELECT id, slug, title, description, election_date FROM election WHERE slug = $1",
            slug
        )
        .fetch_one(&db_pool)
        .await?;
        Ok(record.into())
    }
}
