use crate::{DateTime, Error};
use async_graphql::{Enum, InputObject};
use sqlx::{FromRow, PgPool};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(FromRow, Debug, Clone)]
pub struct Question {
    pub id: uuid::Uuid,
    pub prompt: String,
    pub response_char_limit: Option<i32>,
    pub response_placeholder_text: Option<String>,
    pub allow_anonymous_responses: bool,
    pub embed_id: Option<uuid::Uuid>,
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
    pub sentiment: Option<Sentiment>,
    pub is_locked: bool,
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
}

#[derive(FromRow, Debug, Clone, InputObject)]
pub struct UpsertQuestionSubmissionInput {
    pub id: Option<uuid::Uuid>,
    pub question_id: uuid::Uuid,
    pub candidate_id: Option<Uuid>,
    pub respondent_id: Option<Uuid>,
    pub response: String,
    pub sentiment: Option<Sentiment>,
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

impl Question {
    #[deprecated(note = "Use `insert` and `update` instead")]
    pub async fn upsert(db_pool: &PgPool, input: &UpsertQuestionInput) -> Result<Self, Error> {
        let id = match input.id {
            Some(id) => id,
            None => uuid::Uuid::new_v4(),
        };

        let question = sqlx::query_as!(
            Question,
            r#"
                INSERT INTO question (
                    id,
                    prompt,
                    response_char_limit,
                    response_placeholder_text,
                    allow_anonymous_responses,
                    embed_id
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6
                ) ON CONFLICT (id) DO UPDATE SET
                    prompt = $2,
                    response_char_limit = $3,
                    response_placeholder_text = $4,
                    allow_anonymous_responses = $5,
                    embed_id = $6
                RETURNING *
            "#,
            id,
            input.prompt,
            input.response_char_limit,
            input.response_placeholder_text,
            input.allow_anonymous_responses,
            input.embed_id
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

        if (input.issue_tag_ids).is_some() {
            for issue_tag_id in input.issue_tag_ids.as_ref().unwrap() {
                sqlx::query!(
                    r#"
                    INSERT INTO question_issue_tags (question_id, issue_tag_id)
                    VALUES ($1, $2)
                    ON CONFLICT (question_id, issue_tag_id) DO NOTHING
                "#,
                    id,
                    issue_tag_id
                )
                .execute(db_pool)
                .await?;
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

        let question_submission = sqlx::query_as!(
            QuestionSubmission,
            r#"
                INSERT INTO question_submission (
                    id,
                    question_id,
                    respondent_id,
                    candidate_id,
                    response,
                    sentiment
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6
                ) ON CONFLICT (id) DO UPDATE SET
                    response = $5,
                    sentiment = $6,
                    updated_at = now()
                WHERE question_submission.is_locked <> TRUE
                RETURNING 
                    id,
                    question_id,
                    respondent_id,
                    candidate_id,
                    response,
                    sentiment AS "sentiment:Sentiment",
                    is_locked,
                    created_at,
                    updated_at
            "#,
            id,
            input.question_id,
            input.respondent_id,
            input.candidate_id,
            input.response,
            input.sentiment as Option<Sentiment>
        )
        .fetch_one(db_pool)
        .await?;
        Ok(question_submission)
    }

    pub async fn lock(db_pool: &PgPool, id: uuid::Uuid) -> Result<bool, Error> {
        let _question_submission = sqlx::query_as!(
            QuestionSubmission,
            r#"
                UPDATE question_submission
                SET is_locked = TRUE
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(true)
    }

    pub async fn unlock(db_pool: &PgPool, id: uuid::Uuid) -> Result<bool, Error> {
        let _question_submission = sqlx::query_as!(
            QuestionSubmission,
            r#"
                UPDATE question_submission
                SET is_locked = FALSE
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(true)
    }
}
