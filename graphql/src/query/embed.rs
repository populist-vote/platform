use async_graphql::{Context, Object, Result, SimpleObject, ID};
use auth::AccessTokenClaims;
use db::{Embed, EmbedFilter, EmbedType};
use jsonwebtoken::TokenData;

use crate::{context::ApiContext, guard::OrganizationGuard, is_admin, types::EmbedResult};

#[derive(Default)]
pub struct EmbedQuery;

#[derive(SimpleObject)]
pub struct EmbedsCountResult {
    embed_type: EmbedType,
    embed_count: Option<i64>,
    unique_origin_count: Option<i64>,
    total_deployments: Option<i64>,
    submissions: Option<i64>,
}

#[Object]
impl EmbedQuery {
    #[graphql(
        guard = "OrganizationGuard::new(&organization_id)",
        visible = "is_admin"
    )]
    async fn embeds_activity(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
    ) -> Result<Vec<EmbedsCountResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = sqlx::query_as!(
            EmbedsCountResult,
            r#"
            SELECT
                e.embed_type AS "embed_type:EmbedType",
                COUNT(DISTINCT e.id) AS embed_count,
                COUNT(DISTINCT eo.url) AS unique_origin_count,
                COUNT(eo.url) AS total_deployments,
                COALESCE(COUNT(DISTINCT ps.id), COUNT(DISTINCT qs.id), 0) AS submissions
            FROM
                embed e
            LEFT JOIN
                embed_origin eo ON e.id = eo.embed_id
            LEFT JOIN
                poll p ON (e.attributes->>'pollId')::uuid = p.id
            LEFT JOIN
                poll_submission ps ON p.id = ps.poll_id
            LEFT JOIN
                question q ON (e.attributes->>'questionId')::uuid = q.id
            LEFT JOIN
                question_submission qs ON q.id = qs.question_id
            WHERE
                e.organization_id = $1
            GROUP BY
                e.embed_type;     
        "#,
            uuid::Uuid::parse_str(&organization_id)?,
        )
        .fetch_all(&db_pool)
        .await?;
        Ok(result)
    }

    #[graphql(
        guard = "OrganizationGuard::new(&organization_id)",
        visible = "is_admin"
    )]
    async fn embeds_by_organization(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
        filter: Option<EmbedFilter>,
    ) -> Result<Vec<EmbedResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Embed::find_by_organization_id(
            &db_pool,
            uuid::Uuid::parse_str(&organization_id)?,
            filter.unwrap_or_default(),
        )
        .await?;
        let results = records.into_iter().map(EmbedResult::from).collect();
        Ok(results)
    }

    #[graphql(visible = "is_admin")]
    async fn embed_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<EmbedResult> {
        tracing::debug!("Embed ID: {}", id.as_str());
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Embed::find_by_id(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<AccessTokenClaims>>>() {
            if token_data.claims.organization_id.unwrap_or_default() != record.organization_id {
                return Err("Unauthorized".into());
            }
        }
        Ok(record.into())
    }
}
