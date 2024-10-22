use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::warn;

use crate::{DateTime, Error};

#[derive(FromRow, Debug, Clone)]
pub struct Poll {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub prompt: String,
    pub embed_id: Option<uuid::Uuid>,
    pub allow_anonymous_responses: bool,
    pub allow_write_in_responses: bool,
    pub organization_id: uuid::Uuid,
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
    pub respondent_id: Option<uuid::Uuid>,
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
    pub poll_option_id: Option<uuid::Uuid>,
    pub write_in_response: Option<String>,
}

#[derive(Serialize, Deserialize, InputObject, Debug)]
pub struct UpsertPollInput {
    pub id: Option<uuid::Uuid>,
    pub prompt: Option<String>,
    pub name: Option<String>,
    pub allow_anonymous_responses: Option<bool>,
    pub allow_write_in_responses: Option<bool>,
    pub options: Vec<UpsertPollOptionInput>,
    pub organization_id: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, InputObject, Debug)]
pub struct UpsertPollOptionInput {
    pub id: Option<uuid::Uuid>,
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
                allow_anonymous_responses,
                allow_write_in_responses,
                organization_id
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6
            ) ON CONFLICT (id) DO UPDATE SET
                prompt = EXCLUDED.prompt,
                name = EXCLUDED.name,
                allow_anonymous_responses = EXCLUDED.allow_anonymous_responses,
                allow_write_in_responses = EXCLUDED.allow_write_in_responses
            RETURNING *
            "#,
            id,
            input.prompt,
            input.name,
            input.allow_anonymous_responses,
            input.allow_write_in_responses,
            input.organization_id
        )
        .fetch_one(&mut *tx)
        .await?;

        warn!("input options: {:?}", input.options);
        // Delete any poll options whose IDs are not passed in the input
        sqlx::query!(
            r#"
            DELETE FROM poll_option
            WHERE poll_id = $1
            AND id NOT IN (
                SELECT id FROM UNNEST($2::uuid[])
            )
            "#,
            id,
            &input
                .options
                .iter()
                .filter_map(|option| option.id)
                .collect::<Vec<uuid::Uuid>>()
        )
        .execute(&mut *tx)
        .await?;

        for option in input.options.iter() {
            let option_id = match option.id {
                Some(id) => id,
                None => uuid::Uuid::new_v4(),
            };
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
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(poll)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            Poll,
            r#"
                SELECT * FROM poll 
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(record)
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
