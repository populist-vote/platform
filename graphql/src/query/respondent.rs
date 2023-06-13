use async_graphql::{Context, Object, Result, ID};
use db::{Organization, Respondent};

use crate::context::ApiContext;
use crate::relay;
use crate::types::{OrganizationResult, RespondentResult};

#[derive(Default)]
pub struct RespondentQuery;

#[Object]
impl RespondentQuery {
    async fn respondents_by_organization_id(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<RespondentResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            Respondent,
            r#"
            SELECT r.id, r.name, r.email, r.created_at, r.updated_at FROM respondent r JOIN organization_respondents 
                ON organization_respondents.respondent_id = r.id 
                WHERE organization_respondents.organization_id = $1
            "#,
            uuid::Uuid::parse_str(&organization_id)?
        )
        .fetch_all(&db_pool)
        .await?;

        let results: Vec<RespondentResult> =
            records.into_iter().map(RespondentResult::from).collect();

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    async fn organization_by_slug(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Organization::find_by_slug(&db_pool, slug).await?;

        Ok(record.into())
    }

    async fn organization_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Organization::find_by_id(&db_pool, uuid::Uuid::parse_str(&id)?).await?;

        Ok(record.into())
    }
}
