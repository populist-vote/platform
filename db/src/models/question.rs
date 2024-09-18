use crate::{util::translate::translate_text, DateTime, Error};
use async_graphql::{Enum, InputObject};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use super::enums::{PoliticalScope, RaceType, State};

#[derive(FromRow, Debug, Clone)]
pub struct Question {
    pub id: uuid::Uuid,
    pub prompt: String,
    pub translations: Option<serde_json::Value>,
    pub response_char_limit: Option<i32>,
    pub response_placeholder_text: Option<String>,
    pub allow_anonymous_responses: bool,
    pub embed_id: Option<uuid::Uuid>,
    pub organization_id: uuid::Uuid,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(FromRow, Debug, Clone)]
pub struct QuestionSubmission {
    pub id: uuid::Uuid,
    pub question_id: uuid::Uuid,
    pub respondent_id: Option<uuid::Uuid>,
    pub candidate_id: Option<uuid::Uuid>,
    pub response: String,
    pub editorial: Option<String>,
    pub translations: Option<serde_json::Value>,
    pub sentiment: Option<Sentiment>,
    pub copied_from_id: Option<uuid::Uuid>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(FromRow, Debug, Clone, InputObject)]
pub struct UpsertQuestionInput {
    pub id: Option<uuid::Uuid>,
    pub name: Option<String>,
    pub prompt: Option<String>,
    pub response_char_limit: Option<i32>,
    pub response_placeholder_text: Option<String>,
    pub allow_anonymous_responses: Option<bool>,
    pub embed_id: Option<uuid::Uuid>,
    pub candidate_guide_id: Option<uuid::Uuid>,
    pub issue_tag_ids: Option<Vec<uuid::Uuid>>,
    pub translations: Option<serde_json::Value>,
    pub should_translate: Option<bool>,
    pub organization_id: Option<uuid::Uuid>,
}

#[derive(FromRow, Debug, Clone, InputObject)]
pub struct InsertQuestionInput {
    pub name: Option<String>,
    pub prompt: String,
    pub response_char_limit: Option<i32>,
    pub response_placeholder_text: Option<String>,
    pub allow_anonymous_responses: bool,
    pub embed_id: Option<uuid::Uuid>,
    pub candidate_guide_id: Option<uuid::Uuid>,
    pub issue_tag_ids: Option<Vec<uuid::Uuid>>,
    pub should_translate: Option<bool>,
}

#[derive(FromRow, Debug, Clone, InputObject)]
pub struct UpsertQuestionSubmissionInput {
    pub id: Option<uuid::Uuid>,
    pub question_id: uuid::Uuid,
    pub candidate_id: Option<Uuid>,
    pub respondent_id: Option<Uuid>,
    pub response: String,
    pub editorial: Option<String>,
    pub sentiment: Option<Sentiment>,
    pub translations: Option<serde_json::Value>,
    pub should_translate: Option<bool>,
    pub copied_from_id: Option<uuid::Uuid>,
}

#[derive(Display, Copy, Clone, Eq, PartialEq, Debug, Enum, EnumString, sqlx::Type)]
#[strum(ascii_case_insensitive)]
#[sqlx(type_name = "sentiment", rename_all = "lowercase")]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Unknown,
}

#[derive(Default, Debug, Serialize, Deserialize, InputObject)]
pub struct QuestionSubmissionsFilter {
    query: Option<String>,
    political_scope: Option<PoliticalScope>,
    race_type: Option<RaceType>,
    state: Option<State>,
    county: Option<String>,
}

impl Question {
    pub async fn upsert(db_pool: &PgPool, input: &UpsertQuestionInput) -> Result<Self, Error> {
        let id = match input.id {
            Some(id) => id,
            None => uuid::Uuid::new_v4(),
        };

        let mut question = sqlx::query_as!(
            Question,
            r#"
                INSERT INTO question (
                    id,
                    prompt,
                    response_char_limit,
                    response_placeholder_text,
                    allow_anonymous_responses,
                    embed_id,
                    translations,
                    organization_id
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8
                ) ON CONFLICT (id) DO UPDATE SET
                    prompt = $2,
                    response_char_limit = $3,
                    response_placeholder_text = $4,
                    allow_anonymous_responses = $5,
                    embed_id = $6,
                    translations = $7
                RETURNING *
            "#,
            id,
            input.prompt,
            input.response_char_limit,
            input.response_placeholder_text,
            input.allow_anonymous_responses,
            input.embed_id,
            input.translations,
            input.organization_id
        )
        .fetch_one(db_pool)
        .await?;

        if (input.candidate_guide_id).is_some() {
            sqlx::query!(
                r#"
                INSERT INTO candidate_guide_questions (question_id, candidate_guide_id)
                VALUES ($1, $2)
                ON CONFLICT (candidate_guide_id, question_id) DO NOTHING
            "#,
                id,
                input.candidate_guide_id
            )
            .execute(db_pool)
            .await?;
        }

        // Input must always contain the desired issue_tag_id set, use empty set to remove all
        if let Some(issue_tag_ids) = input.issue_tag_ids.clone() {
            // Delete existing issue tags
            sqlx::query!(
                r#"
                DELETE FROM question_issue_tags
                WHERE question_id = $1
                "#,
                id
            )
            .execute(db_pool)
            .await?;

            // Insert new issue tags
            sqlx::query!(
                r#"
                INSERT INTO question_issue_tags (question_id, issue_tag_id)
                SELECT $1, unnest($2::uuid[])
                ON CONFLICT (question_id, issue_tag_id) DO NOTHING
                "#,
                id,
                &issue_tag_ids
            )
            .execute(db_pool)
            .await?;
        }

        let should_translate = input.should_translate.unwrap_or(false);

        if should_translate {
            let translations = translate_text(&question.prompt, vec!["es", "so", "hmn"]).await;

            if let Ok(translations) = translations {
                let result = sqlx::query!(
                    r#"
                    UPDATE question
                    SET translations = $1
                    WHERE id = $2
                    returning *
                "#,
                    translations,
                    id
                )
                .fetch_one(db_pool)
                .await?;

                question.translations = result.translations;
            }
        }

        Ok(question)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            Question,
            r#"
                SELECT * FROM question WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(record)
    }
}

impl QuestionSubmission {
    pub async fn upsert(
        db_pool: &PgPool,
        input: &UpsertQuestionSubmissionInput,
    ) -> Result<Self, Error> {
        let id = match input.id {
            Some(id) => id,
            None => uuid::Uuid::new_v4(),
        };

        let should_translate = input.should_translate.unwrap_or(false);

        let mut translations = input.translations.clone();

        if should_translate {
            let result = translate_text(&input.response, vec!["es", "so", "hmn"]).await;
            if let Ok(result) = result {
                translations = Some(result);
            }
        }

        let question_submission = sqlx::query_as!(
            QuestionSubmission,
            r#"
                INSERT INTO question_submission (
                    id,
                    question_id,
                    respondent_id,
                    candidate_id,
                    response,
                    sentiment,
                    translations,
                    editorial,
                    copied_from_id
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9
                ) ON CONFLICT (id) DO UPDATE SET
                    response = $5,
                    sentiment = $6,
                    translations = $7,
                    editorial = $8,
                    updated_at = now(),
                    copied_from_id = $9
                RETURNING 
                    id,
                    question_id,
                    respondent_id,
                    candidate_id,
                    response,
                    editorial,
                    translations,
                    sentiment AS "sentiment:Sentiment",
                    copied_from_id,
                    created_at,
                    updated_at
            "#,
            id,
            input.question_id,
            input.respondent_id,
            input.candidate_id,
            input.response,
            input.sentiment as Option<Sentiment>,
            translations,
            input.editorial,
            input.copied_from_id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(question_submission)
    }

    pub async fn filter(
        db_pool: &PgPool,
        organization_id: uuid::Uuid,
        filter: QuestionSubmissionsFilter,
    ) -> Result<Vec<Self>, Error> {
        let search_query = filter.query.to_owned().unwrap_or_default();

        let records = sqlx::query_as!(
            QuestionSubmission,
            r#"
               SELECT DISTINCT ON (qs.candidate_id, qs.question_id, qs.response)
                  qs.id,
                  qs.question_id,
                  respondent_id,
                  qs.candidate_id,
                  response,
                  editorial,
                  qs.translations,
                  sentiment AS "sentiment: Sentiment",
                  copied_from_id,
                  qs.created_at,
                  qs.updated_at
                FROM question_submission qs
                JOIN question q ON qs.question_id = q.id
                JOIN candidate_guide_questions cgq ON qs.question_id = cgq.question_id
                JOIN candidate_guide cg ON cg.id = cgq.candidate_guide_id
                JOIN candidate_guide_races cgr ON cg.id = cgr.candidate_guide_id
                JOIN race r ON cgr.race_id = r.id
                JOIN office o ON r.office_id = o.id
                JOIN politician p ON qs.candidate_id = p.id
                JOIN race_candidates rc ON rc.candidate_id = p.id AND rc.race_id = r.id,
                to_tsvector(
                    r.title || o.title || ' ' || o.name || ' ' || COALESCE(o.subtitle, '') || ' ' || COALESCE(o.office_type, '') || ' ' || COALESCE(o.district, '') || ' ' || COALESCE(o.hospital_district, '') || ' ' || COALESCE(o.school_district, '') || ' ' || COALESCE(o.state::text, '') || ' ' || COALESCE(o.county, '') || ' ' || COALESCE(o.municipality, '') || ' ' || COALESCE(p.full_name, '')
                  ) document,
                websearch_to_tsquery($2) AS query
                WHERE q.organization_id = $1::uuid
                  AND(($2::text = '') IS NOT FALSE OR query @@ document)
                  AND ($3::race_type IS NULL OR r.race_type = $3::race_type)
                  AND ($4::political_scope IS NULL OR o.political_scope = $4::political_scope)
                  AND ($5::state IS NULL OR o.state = $5::state)
                  AND ($6::text IS NULL OR o.county = $6::text)
                LIMIT 250
                            "#,
            organization_id,
            search_query,
            filter.race_type as Option<RaceType>,
            filter.political_scope as Option<PoliticalScope>,
            filter.state as Option<State>,
            filter.county
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }
}
