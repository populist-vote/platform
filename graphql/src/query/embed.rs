use crate::{context::ApiContext, guard::OrganizationGuard, is_admin, types::EmbedResult};
use async_graphql::{Context, Object, Result, SimpleObject, ID};
use auth::AccessTokenClaims;
use chrono::Utc;
use db::{Embed, EmbedFilter, EmbedType, OrganizationRoleType};
use jsonwebtoken::TokenData;

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

#[derive(SimpleObject)]
pub struct EnhancedEmbedOriginResult {
    embed_id: uuid::Uuid,
    embed_type: EmbedType,
    name: String,
    url: String,
    last_ping_at: chrono::DateTime<Utc>,
}

#[Object]
impl EmbedQuery {
    #[graphql(
        guard = "OrganizationGuard::new(&organization_id, &OrganizationRoleType::ReadOnly)",
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
        guard = "OrganizationGuard::new(&organization_id, &OrganizationRoleType::ReadOnly)",
        visible = "is_admin"
    )]
    async fn recent_deployments(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
        limit: Option<i64>,
    ) -> Result<Vec<EnhancedEmbedOriginResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            EnhancedEmbedOriginResult,
            r#"
                SELECT
                    e.id AS embed_id,
                    e.embed_type AS "embed_type:EmbedType",
                    e.name,
                    eo.url,
                    eo.last_ping_at
                FROM
                    embed_origin eo
                JOIN
                    embed e ON eo.embed_id = e.id
                WHERE
                    e.organization_id = $1
                ORDER BY
                    eo.last_ping_at DESC
                LIMIT $2;
            "#,
            uuid::Uuid::parse_str(&organization_id)?,
            limit.unwrap_or(6),
        )
        .fetch_all(&db_pool)
        .await?;

        let results = records
            .into_iter()
            .map(EnhancedEmbedOriginResult::from)
            .collect();

        Ok(results)
    }

    #[graphql(
        guard = "OrganizationGuard::new(&organization_id, &OrganizationRoleType::ReadOnly)",
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
            if token_data
                .claims
                .organizations
                .iter()
                .any(|role| role.organization_id == record.organization_id)
            {
                return Ok(record.into());
            }
        }
        Ok(record.into())
    }

    #[graphql(
        guard = "OrganizationGuard::new(&organization_id, &OrganizationRoleType::ReadOnly)",
        visible = "is_admin"
    )]
    async fn total_candidate_guide_submissions(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
    ) -> Result<i32> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = sqlx::query!(
            r#"
            WITH guide_averages AS (
                SELECT
                    cg.id AS candidate_guide_id,
                    COUNT(qs.id) / COUNT(DISTINCT q.id) AS average_submissions_per_question
                FROM
                    candidate_guide cg
                JOIN
                    candidate_guide_questions cgq ON cg.id = cgq.candidate_guide_id
                JOIN
                    question q ON cgq.question_id = q.id
                LEFT JOIN
                    question_submission qs ON q.id = qs.question_id
                WHERE cg.organization_id = $1
                GROUP BY
                    cg.id
            )
            SELECT
                COALESCE(SUM(average_submissions_per_question)::integer, 0) AS total_average_submissions
            FROM
                guide_averages;
        "#,
            uuid::Uuid::parse_str(&organization_id)?,
        )
        .fetch_one(&db_pool)
        .await?;

        let total_average_submissions = result.total_average_submissions.unwrap_or_default();
        Ok(total_average_submissions)
    }
}
