use crate::{
    is_admin,
    types::{QuestionResult, QuestionSubmissionResult},
};
use async_graphql::{Context, Object, Result, SimpleObject, ID};
use async_openai::types::CreateCompletionRequestArgs;
use db::{
    models::{question::UpsertQuestionInput, respondent::UpsertRespondentInput},
    QuestionSubmission, Sentiment, UpsertQuestionSubmissionInput,
};

use crate::context::ApiContext;

#[derive(Default)]
pub struct QuestionMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteQuestionResult {
    id: ID,
}

#[Object]
impl QuestionMutation {
    async fn upsert_question(
        &self,
        ctx: &Context<'_>,
        input: UpsertQuestionInput,
    ) -> Result<QuestionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_question = db::Question::upsert(&db_pool, &input).await?;
        Ok(new_question.into())
    }

    async fn delete_question(&self, ctx: &Context<'_>, id: ID) -> Result<DeleteQuestionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        sqlx::query!(
            r#"
                DELETE FROM question WHERE id = $1
            "#,
            uuid::Uuid::parse_str(&id)?
        )
        .execute(&db_pool)
        .await?;
        Ok(DeleteQuestionResult { id })
    }
}

#[derive(Default)]
pub struct QuestionSubmissionMutation;

#[Object]
impl QuestionSubmissionMutation {
    async fn upsert_question_submission(
        &self,
        ctx: &Context<'_>,
        respondent_input: Option<UpsertRespondentInput>,
        question_submission_input: UpsertQuestionSubmissionInput,
    ) -> Result<QuestionSubmissionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let respondent = match respondent_input {
            Some(respondent_input) => {
                Some(db::Respondent::upsert(&db_pool, &respondent_input).await?)
            }
            None => None,
        };

        // Use OpenAI to determine the sentiment of the response
        let client = async_openai::Client::new();
        let prompt = format!(
            r#"
                Classify the sentiment in this response as either positive, negative, or neutral:
                Response: {}"#,
            question_submission_input.response
        );
        let request = CreateCompletionRequestArgs::default()
            .model("gpt-3.5-turbo-instruct")
            .prompt(prompt)
            .max_tokens(40_u16)
            .build()?;
        let response = client.completions().create(request).await;
        let sentiment = match response {
            Ok(response) => {
                let trimmed_response_text = &response
                    .choices
                    .first()
                    .unwrap()
                    .text
                    .as_str()
                    .trim()
                    .to_lowercase();
                if trimmed_response_text.contains("positive") {
                    Sentiment::Positive
                } else if trimmed_response_text.contains("negative") {
                    Sentiment::Negative
                } else if trimmed_response_text.contains("neutral") {
                    Sentiment::Neutral
                } else {
                    Sentiment::Unknown
                }
            }
            Err(err) => {
                tracing::error!("Error classifying sentiment with OpenAI: {:?}", err);
                Sentiment::Unknown
            }
        };

        let question_submission_input = UpsertQuestionSubmissionInput {
            respondent_id: respondent.map(|r| r.id),
            sentiment: Some(sentiment),
            ..question_submission_input
        };

        let question = db::QuestionSubmission::upsert(&db_pool, &question_submission_input).await?;
        Ok(question.into())
    }

    async fn copy_question_submission(
        &self,
        ctx: &Context<'_>,
        question_submission_id: ID,
        target_question_id: ID,
    ) -> Result<QuestionSubmissionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let question_submission_id = uuid::Uuid::parse_str(&question_submission_id)?;
        let target_question_id = uuid::Uuid::parse_str(&target_question_id)?;

        let result = sqlx::query_as!(
            QuestionSubmission,
            r#"
                INSERT INTO question_submission (
                    question_id,
                    respondent_id,
                    candidate_id,
                    response,
                    editorial,
                    translations,
                    sentiment,
                    copied_from_id,
                    created_at,
                    updated_at
                )
                SELECT
                    $2 AS question_id,  -- This is the new question_id you want to associate the copy with
                    qs.respondent_id,
                    qs.candidate_id,
                    qs.response,
                    qs.editorial,
                    qs.translations,
                    qs.sentiment,
                    $1 AS copied_from_id,  -- This is the original question_submission_id you want to copy
                    NOW(),  -- Set the creation timestamp to now
                    NOW()   -- Set the updated timestamp to now
                FROM
                    question_submission qs
                WHERE
                    qs.id = $1
                RETURNING 
                    id,
                    question_id,
                    respondent_id,
                    candidate_id,
                    response,
                    editorial,
                    translations,
                    sentiment AS "sentiment: Sentiment",
                    copied_from_id,
                    created_at,
                    updated_at
            "#,
            target_question_id,
            question_submission_id
        ).fetch_one(&db_pool).await?;

        Ok(result.into())
    }
}
