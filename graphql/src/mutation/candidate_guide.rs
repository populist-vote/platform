use crate::{context::ApiContext, types::CandidateGuideResult};
use async_graphql::{Context, Object, Result, ID};
use auth::{create_random_token, AccessTokenClaims};
use db::{
    models::candidate_guide::{CandidateGuide, UpsertCandidateGuideInput},
    EmbedType, UpsertEmbedInput,
};
use jsonwebtoken::TokenData;

#[derive(Default)]
pub struct CandidateGuideMutation;

#[Object]
impl CandidateGuideMutation {
    async fn upsert_candidate_guide(
        &self,
        ctx: &Context<'_>,
        input: UpsertCandidateGuideInput,
    ) -> Result<CandidateGuideResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user = ctx.data::<Option<TokenData<AccessTokenClaims>>>().unwrap();
        let organization_id = user.as_ref().unwrap().claims.organization_id.unwrap();
        let input = UpsertCandidateGuideInput {
            user_id: Some(user.as_ref().unwrap().claims.sub),
            organization_id: Some(organization_id),
            ..input
        };
        let upsert = CandidateGuide::upsert(&db_pool, &input).await?;

        // Created embeds of type candidate_guide for each race associated with the candidate guide
        if input.race_ids.is_some() {
            for race_id in input.race_ids.unwrap() {
                let embed_input = UpsertEmbedInput {
                    id: None,
                    organization_id: Some(organization_id),
                    name: upsert.name.clone(),
                    description: None,
                    embed_type: Some(EmbedType::CandidateGuide),
                    attributes: Some(serde_json::json!({
                        "candidate_guide_id": upsert.id,
                        "race_id": race_id
                    })),
                };
                let updated_by = user.as_ref().unwrap().claims.sub;
                db::models::embed::Embed::upsert(&db_pool, &embed_input, &updated_by).await?;
            }
        }

        Ok(upsert.into())
    }

    async fn remove_candidate_guide_race(
        &self,
        ctx: &Context<'_>,
        candidate_guide_id: ID,
        race_id: ID,
    ) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = sqlx::query!(
            r#"
            WITH deleted_guide_race AS (
                DELETE FROM candidate_guide_races
                WHERE candidate_guide_id = $1
                    AND race_id = $2
            ) DELETE FROM embed
            WHERE embed_type = 'candidate_guide'
                AND attributes ->> 'candidate_guide_id' = $1::text
                AND attributes ->> 'race_id' = $2::text
        "#,
            uuid::Uuid::parse_str(candidate_guide_id.as_str())?,
            uuid::Uuid::parse_str(race_id.as_str())?,
        )
        .execute(&db_pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    async fn generate_intake_token_link(
        &self,
        ctx: &Context<'_>,
        candidate_guide_id: ID,
        politician_id: ID,
    ) -> Result<String> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let token = create_random_token().unwrap();
        sqlx::query!(
            r#"
            UPDATE politician SET intake_token = $1 WHERE id = $2
        "#,
            token,
            uuid::Uuid::parse_str(&politician_id)?,
        )
        .execute(&db_pool)
        .await?;

        let url = format!(
            "{}/intakes/candidate-guides/{}?token={}",
            config::Config::default().web_app_url,
            candidate_guide_id.to_string(),
            token
        );

        Ok(url)
    }

    async fn delete_candidate_guide(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        CandidateGuide::delete(&db_pool, uuid::Uuid::parse_str(id.as_str()).unwrap()).await?;
        Ok(true)
    }
}
