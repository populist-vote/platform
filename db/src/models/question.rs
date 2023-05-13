use async_graphql::InputObject;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{DateTime, Error};

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
    pub response: String,
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
}

#[derive(FromRow, Debug, Clone, InputObject)]
pub struct UpsertQuestionSubmissionInput {
    pub id: Option<uuid::Uuid>,
    pub question_id: uuid::Uuid,
    pub respondent_id: Option<Uuid>,
    pub response: String,
}

impl Question {
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
                    response
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4
                ) ON CONFLICT (id) DO UPDATE SET
                    response = $4,
                    updated_at = now()
                RETURNING *
            "#,
            id,
            input.question_id,
            input.respondent_id,
            input.response
        )
        .fetch_one(db_pool)
        .await?;
        Ok(question_submission)
    }
}
