use crate::{
    context::ApiContext,
    types::{
        UpsertVotingGuideCandidateInput, UpsertVotingGuideInput, VotingGuideCandidateResult,
        VotingGuideResult,
    },
};
use async_graphql::{Context, Object, Result, SimpleObject};
use auth::Claims;
use db::models::voting_guide::VotingGuide;
use jsonwebtoken::TokenData;
use uuid::Uuid;

#[derive(Default)]
pub struct VotingGuideMutation;

#[derive(SimpleObject)]
struct DeleteVotingGuideResult {
    id: String,
}

#[Object]
impl VotingGuideMutation {
    async fn upsert_voting_guide(
        &self,
        ctx: &Context<'_>,
        input: UpsertVotingGuideInput,
    ) -> Result<VotingGuideResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;
        let new_record = sqlx::query_as!(
            VotingGuide,
            r#"
            INSERT INTO voting_guide (id, user_id, title, description)
                VALUES($1, $2, $3, $4) ON CONFLICT (id)
                DO
                UPDATE
                SET
                    title = $3,
                    description = $4
            RETURNING
                id,
                user_id,
                title,
                description,
                created_at,
                updated_at
        "#,
            Uuid::parse_str(input.id.unwrap_or_default().as_str()).unwrap_or(Uuid::new_v4()),
            user_id,
            input.title,
            input.description,
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(new_record.into())
    }

    async fn upsert_voting_guide_candidate(
        &self,
        ctx: &Context<'_>,
        input: UpsertVotingGuideCandidateInput,
    ) -> Result<VotingGuideCandidateResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query!(
            r#"
            INSERT INTO voting_guide_candidates (voting_guide_id, candidate_id, is_endorsement, note)
                VALUES($1, $2, $3, $4) ON CONFLICT (voting_guide_id, candidate_id)
                DO
                UPDATE
                SET
                    is_endorsement = $3,
                    note = $4
            RETURNING
                candidate_id,
                is_endorsement,
                note
        "#,
            Uuid::parse_str(input.voting_guide_id.as_str()).unwrap(),
            Uuid::parse_str(input.candidate_id.as_str()).unwrap(),
            input.is_endorsement,
            input.note,
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(VotingGuideCandidateResult {
            candidate_id: record.candidate_id.into(),
            is_endorsement: record.is_endorsement,
            note: record.note,
        })
    }
}
