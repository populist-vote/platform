use crate::{context::ApiContext, types::CandidateGuideResult};
use async_graphql::{Context, Object, Result, ID};
use auth::AccessTokenClaims;
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
        let organization_id = input.organization_id;
        let input = UpsertCandidateGuideInput {
            user_id: Some(user.as_ref().unwrap().claims.sub),
            organization_id,
            ..input
        };
        let upsert = CandidateGuide::upsert(&db_pool, &input).await?;

        // Created embeds of type candidate_guide for each race associated with the candidate guide
        if input.race_ids.is_some() {
            for race_id in input.race_ids.unwrap() {
                let embed_input = UpsertEmbedInput {
                    id: None,
                    organization_id,
                    name: upsert.name.clone(),
                    description: None,
                    embed_type: Some(EmbedType::CandidateGuide),
                    attributes: Some(serde_json::json!({
                        "candidateGuideId": upsert.id,
                        "raceId": race_id
                    })),
                };
                let updated_by = user.as_ref().unwrap().claims.sub;
                db::models::embed::Embed::upsert(&db_pool, &embed_input, &updated_by).await?;
            }
        }

        Ok(upsert.into())
    }

    async fn open_all_candidate_guide_submissions(
        &self,
        ctx: &Context<'_>,
        candidate_guide_id: ID,
    ) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = db::models::candidate_guide::CandidateGuide::open_all_submissions(
            &db_pool,
            uuid::Uuid::parse_str(candidate_guide_id.as_str())?,
        )
        .await?;

        Ok(result)
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
                AND attributes ->> 'candidateGuideId' = $1::text
                AND attributes ->> 'raceId' = $2::text
        "#,
            uuid::Uuid::parse_str(candidate_guide_id.as_str())?,
            uuid::Uuid::parse_str(race_id.as_str())?,
        )
        .execute(&db_pool)
        .await?;

        Ok(result.rows_affected() == 2)
    }

    async fn generate_intake_token_link(
        &self,
        ctx: &Context<'_>,
        candidate_guide_id: ID,
        race_id: ID,
        politician_id: ID,
    ) -> Result<String> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_politician = sqlx::query!(
            r#"
            UPDATE politician
            SET intake_token = COALESCE(intake_token, encode(gen_random_bytes(32), 'hex'))
            WHERE id = $1
            RETURNING intake_token
        "#,
            uuid::Uuid::parse_str(&politician_id)?,
        )
        .fetch_one(&db_pool)
        .await?;

        let url = format!(
            "{}/intakes/candidate-guides/{}?raceId={}&token={}",
            config::Config::default().web_app_url,
            *candidate_guide_id,
            *race_id,
            updated_politician.intake_token.unwrap_or_default()
        );

        Ok(url)
    }

    // We should expand this fn to allow clients to download fine grained data for these
    // candidate guides, intakes, etc.
    /// Download all candidate guide data as a CSV string, must be converted to CSV file by client
    async fn download_all_candidate_guide_data(
        &self,
        ctx: &Context<'_>,
        candidate_guide_id: ID,
        race_id: Option<ID>,
    ) -> Result<String> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let records = sqlx::query!(
            r#"
            WITH races AS (
                SELECT
                    id AS populist_race_id,
                    title AS race_title,
                    cgr.were_candidates_emailed
                FROM
                    race
                    JOIN candidate_guide_races cgr ON cgr.race_id = race.id
                WHERE
                    cgr.candidate_guide_id = $1
                AND ($2::uuid IS NULL OR race.id = $2::uuid)
            ),
            politicians AS (
                SELECT
                    r.*,
                    p.first_name,
                    p.middle_name,
                    p.last_name,
                    p.preferred_name,
                    p.suffix,
                    p.email AS email,
                    p.id AS politician_id,
                    p.intake_token
                FROM
                    races r
                    JOIN race_candidates rc ON rc.race_id = r.populist_race_id
                    JOIN politician p ON rc.candidate_id = p.id
            ),
            update_politician_intake_tokens AS (
                UPDATE
                    politician
                SET
                    intake_token = encode(gen_random_bytes(32), 'hex')
                FROM
                    politicians
                WHERE
                    politician.id = politicians.politician_id
                    AND politician.intake_token IS NULL
                RETURNING politician.id, politician.intake_token
            ),
            last_submissions AS (
                SELECT
                    qs.candidate_id,
                    MAX(qs.updated_at) AS last_submission
                FROM
                    question_submission qs
                    JOIN candidate_guide_questions cgq ON qs.question_id = cgq.question_id
                WHERE
                    cgq.candidate_guide_id = $1
                GROUP BY
                    qs.candidate_id
            )
            SELECT
                r.populist_race_id AS race_id,
                r.*, 
                p.first_name,
                p.middle_name,
                p.last_name,
                p.preferred_name,
                p.suffix,
                p.email AS email,
                p.id AS politician_id, 
                COALESCE(upt.intake_token, p.intake_token) AS intake_token,
                ls.last_submission
            FROM
                races r
                JOIN race_candidates rc ON rc.race_id = r.populist_race_id
                JOIN politician p ON rc.candidate_id = p.id
                LEFT JOIN update_politician_intake_tokens upt ON upt.id = p.id
                LEFT JOIN last_submissions ls ON p.id = ls.candidate_id
            WHERE
                ($2::uuid IS NULL OR r.populist_race_id = $2::uuid);
        "#,
            uuid::Uuid::parse_str(&candidate_guide_id)?,
            race_id
                .map(|id| uuid::Uuid::parse_str(id.as_str()))
                .transpose()?,
        )
        .fetch_all(&db_pool)
        .await?;

        let mut csv_string = Vec::new();
        {
            let mut wtr = csv::Writer::from_writer(&mut csv_string);
            wtr.write_record([
                "race_title",
                "first_name",
                "middle_name",
                "last_name",
                "preferred_name",
                "suffix",
                "full_name",
                "email",
                "form_link",
                "was_candidate_emailed",
                "last_submission",
            ])?;
            for record in records {
                let full_name = format!(
                    "{first_name} {last_name} {suffix}",
                    first_name = &record.preferred_name.as_ref().unwrap_or(&record.first_name),
                    last_name = &record.last_name,
                    suffix = &record.suffix.as_ref().unwrap_or(&"".to_string())
                )
                .trim_end()
                .to_string();
                let form_link = if let Some(intake_token) = record.intake_token {
                    format!(
                        "{}/intakes/candidate-guides/{}?raceId={}&token={}",
                        config::Config::default().web_app_url,
                        *candidate_guide_id,
                        record.race_id,
                        intake_token
                    )
                } else {
                    "".to_string()
                };
                wtr.write_record(&[
                    record.race_title,
                    record.first_name,
                    record.middle_name.unwrap_or_default(),
                    record.last_name,
                    record.preferred_name.unwrap_or_default(),
                    record.suffix.unwrap_or_default(),
                    full_name,
                    record.email.unwrap_or_default(),
                    form_link,
                    record
                        .were_candidates_emailed
                        .map(|b| b.to_string())
                        .unwrap_or_default(),
                    record
                        .last_submission
                        .map(|d| d.to_string())
                        .unwrap_or_default(),
                ])?;
            }
            wtr.flush()?;
        }
        Ok(String::from_utf8(csv_string).unwrap())
    }

    async fn delete_candidate_guide(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        CandidateGuide::delete(&db_pool, uuid::Uuid::parse_str(id.as_str()).unwrap()).await?;
        Ok(true)
    }
}
