use async_graphql::{Context, FieldResult, Object};
use db::{Election, ElectionSearchInput};

use crate::{context::ApiContext, types::ElectionResult};

#[derive(Default)]
pub struct ElectionQuery;

#[Object]
impl ElectionQuery {
    async fn all_elections(&self, ctx: &Context<'_>) -> FieldResult<Vec<ElectionResult>> {
        // let token = ctx.data_unchecked::<Option<String>>();
        // let auth_claim = auth::validate_token(token.as_ref().unwrap()).await;
        // println!("{:?}", token);
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Election::index(&db_pool).await?;
        let results = records.into_iter().map(ElectionResult::from).collect();
        Ok(results)
    }

    async fn elections(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by slug or title")] search: ElectionSearchInput,
    ) -> FieldResult<Vec<ElectionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Election::search(&db_pool, &search).await?;
        let results = records.into_iter().map(ElectionResult::from).collect();
        Ok(results)
    }

    // Need to think about this.
    // User is going to only want to see relevant election, based on locale
    // Perhaps to start, implement an upcoming_election_by_state() resolver
    async fn upcoming_elections(&self, ctx: &Context<'_>) -> FieldResult<Vec<ElectionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(Election,
            "SELECT id, slug, title, description, election_date FROM election WHERE election_date > NOW()", )
            .fetch_all(&db_pool)
            .await?;
        let results = records.into_iter().map(ElectionResult::from).collect();
        Ok(results)
    }

    async fn election_by_id(&self, ctx: &Context<'_>, id: String) -> FieldResult<ElectionResult> {
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
}
