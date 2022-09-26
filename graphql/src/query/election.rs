use async_graphql::{Context, FieldResult, Object, Result, ID};
use auth::Claims;
use db::{models::enums::State, Election, ElectionSearchInput};
use jsonwebtoken::TokenData;

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

    async fn elections_by_user_state(&self, ctx: &Context<'_>) -> Result<Vec<ElectionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let token = ctx.data::<Option<TokenData<Claims>>>();

        if let Some(token_data) = token.unwrap() {
            let users_state = sqlx::query!(
                r#"
                SELECT
                    a.state AS "state:State"
                FROM
                    address a
                    JOIN user_profile up ON user_id = $1
                WHERE
                    up.user_id = $1 AND 
                    up.address_id = a.id
                "#,
                token_data.claims.sub
            )
            .fetch_one(&db_pool)
            .await?
            .state;

            let records = sqlx::query_as!(
                Election,
                r#"SELECT
                id,
                slug,
                title,
                description,
                state AS "state:State",
                election_date
            FROM
                election
            WHERE state = $1 OR state IS NULL
            ORDER BY
                election_date ASC
            LIMIT 1"#,
                users_state as State
            )
            .fetch_all(&db_pool)
            .await?;

            Ok(records.into_iter().map(ElectionResult::from).collect())
        } else {
            tracing::debug!("No elections found with user address data");
            Err("No user address data found".into())
        }
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
                state AS "state:State",
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
            r#"SELECT id, slug, title, description, state AS "state:State", election_date FROM election WHERE id = $1"#,
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
            r#"SELECT id, slug, title, description, state AS "state:State", election_date FROM election WHERE slug = $1"#,
            slug
        )
        .fetch_one(&db_pool)
        .await?;
        Ok(record.into())
    }
}
