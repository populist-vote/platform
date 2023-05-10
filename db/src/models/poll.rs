use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::{DateTime, Error};

#[derive(FromRow, Debug, Clone)]
pub struct Poll {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub prompt: String,
    pub embed_id: Option<uuid::Uuid>,
    pub allow_anonymous_responses: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(FromRow, Debug, Clone)]
pub struct PollOption {
    pub id: uuid::Uuid,
    pub poll_id: uuid::Uuid,
    pub option_text: String,
    pub is_write_in: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(FromRow, Debug, Clone)]
pub struct PollSubmission {
    pub id: uuid::Uuid,
    pub poll_id: uuid::Uuid,
    pub respondent_id: uuid::Uuid,
    pub poll_option_id: uuid::Uuid,
    pub write_in_response: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(FromRow, Debug, Clone, InputObject)]
pub struct UpsertPollSubmissionInput {
    pub id: Option<uuid::Uuid>,
    pub poll_id: uuid::Uuid,
    pub respondent_id: Option<uuid::Uuid>,
    pub poll_option_id: uuid::Uuid,
    pub write_in_response: Option<String>,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct UpsertPollInput {
    pub id: Option<uuid::Uuid>,
    pub prompt: String,
    pub name: Option<String>,
    pub allow_anonymous_responses: Option<bool>,
    pub options: Vec<UpsertPollOptionInput>,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct UpsertPollOptionInput {
    pub option_text: String,
    pub is_write_in: Option<bool>,
}

impl Poll {
    pub async fn upsert(db_pool: &PgPool, input: &UpsertPollInput) -> Result<Self, Error> {
        let mut tx = db_pool.begin().await?;
        let id = match input.id {
            Some(id) => id,
            None => uuid::Uuid::new_v4(),
        };

        let poll = sqlx::query_as!(
            Poll,
            r#"
            INSERT INTO poll (
                id,
                prompt,
                name,
                allow_anonymous_responses
            ) VALUES (
                $1,
                $2,
                $3,
                $4
            ) ON CONFLICT (id) DO UPDATE SET
                prompt = EXCLUDED.prompt
            RETURNING *
            "#,
            id,
            input.prompt,
            input.name,
            input.allow_anonymous_responses.unwrap_or(false),
        )
        .fetch_one(&mut tx)
        .await?;

        tracing::info!("Upserted poll {:?}", poll);

        for option in input.options.iter() {
            let option_id = uuid::Uuid::new_v4();
            let is_write_in = option.is_write_in.unwrap_or(false);
            sqlx::query!(
                r#"
                INSERT INTO poll_option (
                    id,
                    poll_id,
                    option_text,
                    is_write_in
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4
                ) ON CONFLICT (id) DO UPDATE SET
                    option_text = EXCLUDED.option_text,
                    is_write_in = EXCLUDED.is_write_in
                "#,
                option_id,
                id,
                option.option_text,
                is_write_in,
            )
            .execute(&mut tx)
            .await?;
        }

        tx.commit().await?;

        Ok(poll)
    }
}

impl PollSubmission {
    pub async fn upsert(
        db_pool: &PgPool,
        input: &UpsertPollSubmissionInput,
    ) -> Result<Self, Error> {
        let id = match input.id {
            Some(id) => id,
            None => uuid::Uuid::new_v4(),
        };
        let poll_submission = sqlx::query_as!(
            PollSubmission,
            r#"
            INSERT INTO poll_submission (
                id,
                poll_id,
                poll_option_id,
                respondent_id
            ) VALUES (
                $1,
                $2,
                $3,
                $4
            ) ON CONFLICT (id) DO UPDATE SET
                poll_option_id = EXCLUDED.poll_option_id
            RETURNING *
            "#,
            id,
            input.poll_id,
            input.poll_option_id,
            input.respondent_id,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(poll_submission)
    }
}
