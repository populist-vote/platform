use async_graphql::{Context, Object, Result, ID};
use db::models::voting_guide::VotingGuide;
use uuid::Uuid;

use crate::{context::ApiContext, types::VotingGuideResult};

#[derive(Default)]
pub struct VotingGuideQuery;

#[Object]
impl VotingGuideQuery {
    async fn voting_guides_by_ids(
        &self,
        ctx: &Context<'_>,
        ids: Vec<ID>,
    ) -> Result<Vec<VotingGuideResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let ids: Vec<Uuid> = ids
            .into_iter()
            .map(|id| Uuid::parse_str(id.as_str()).unwrap())
            .collect();

        let records = sqlx::query_as!(
            VotingGuide,
            r#"
            SELECT
                id,
                user_id,
                election_id,
                title,
                description,
                created_at,
                updated_at
            FROM
                voting_guide
            WHERE
                id = ANY ($1)
            "#,
            &ids,
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(records.into_iter().map(|record| record.into()).collect())
    }

    async fn voting_guide_by_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Voting guide id")] id: ID,
    ) -> Result<VotingGuideResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = VotingGuide::find_by_id(&db_pool, uuid::Uuid::parse_str(&id).unwrap()).await?;

        Ok(record.into())
    }

    async fn voting_guides_by_user_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "User id")] user_id: ID,
    ) -> Result<Vec<VotingGuideResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records =
            VotingGuide::find_by_user_id(&db_pool, uuid::Uuid::parse_str(&user_id).unwrap())
                .await?;

        Ok(records.into_iter().map(|record| record.into()).collect())
    }

    /// Returns a single voting guide for the given election and user
    async fn election_voting_guide_by_user_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Election ID")] election_id: ID,
        #[graphql(desc = "User ID")] user_id: ID,
    ) -> Result<Option<VotingGuideResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            VotingGuide,
            r#"
            SELECT
                    id,
                    user_id,
                    election_id,
                    title,
                    description,
                    created_at,
                    updated_at
                FROM
                    voting_guide
                WHERE
                    election_id = $1 AND
                    user_id = $2

        "#,
            uuid::Uuid::parse_str(&election_id).unwrap(),
            uuid::Uuid::parse_str(&user_id).unwrap()
        )
        .fetch_optional(&db_pool)
        .await?;

        Ok(record.map(|record| record.into()))
    }
}
