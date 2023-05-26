use crate::context::ApiContext;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{DateTime, Question, QuestionSubmission, Respondent, Sentiment};

use super::SubmissionCountByDateResult;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct QuestionResult {
    id: ID,
    prompt: String,
    response_char_limit: Option<i32>,
    response_placeholder_text: Option<String>,
    allow_anonymous_responses: bool,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct QuestionSubmissionResult {
    id: ID,
    respondent_id: Option<ID>,
    response: String,
    sentiment: Option<Sentiment>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ExternalUserResult {
    name: String,
    email: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct RespondentResult {
    name: String,
    email: String,
}

#[derive(SimpleObject, Debug, Clone, sqlx::FromRow)]
pub struct CommonWordsResult {
    word: String,
    count: i32,
}

#[derive(SimpleObject, Debug, Clone, sqlx::FromRow)]
pub struct SentimentCountResult {
    sentiment: Sentiment,
    count: i64,
}

#[ComplexObject]
impl QuestionResult {
    async fn submissions(&self, ctx: &Context<'_>) -> Result<Vec<QuestionSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submissions = sqlx::query_as!(
            QuestionSubmission,
            r#"
                SELECT
                  id,
                  question_id,
                  respondent_id,
                  response,
                  sentiment AS "sentiment: Sentiment",
                  created_at,
                  updated_at
                FROM question_submission
                WHERE question_id = $1
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(submissions.into_iter().map(|s| s.into()).collect())
    }

    async fn submission_count_by_date(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<SubmissionCountByDateResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submission_count_by_date = sqlx::query!(
            r#"
                SELECT
                  date_trunc('day', created_at) AS date,
                  COUNT(*) AS count
                FROM question_submission
                WHERE question_id = $1
                GROUP BY date
                ORDER BY date
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(submission_count_by_date
            .into_iter()
            .filter(|s| s.date.is_some())
            .map(|s| SubmissionCountByDateResult {
                date: s.date.unwrap(),
                count: s.count.unwrap(),
            })
            .collect())
    }

    async fn common_words(&self, ctx: &Context<'_>) -> Result<Vec<CommonWordsResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let query = format!(
            r#"
                SELECT word, ndoc AS count
                FROM ts_stat($$
                    SELECT to_tsvector('ts.english_simple', response)
                    FROM (
                        SELECT response
                        FROM question_submission
                        WHERE question_id = '{}'
                    ) AS qs
                $$) AS stats
                WHERE ndoc > 1
                ORDER BY ndoc DESC
                LIMIT 10;
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap()
        );
        let common_words = sqlx::query_as(&query).fetch_all(&db_pool).await?;
        Ok(common_words)
    }

    async fn sentiment_counts(&self, ctx: &Context<'_>) -> Result<Vec<SentimentCountResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let sentiment_counts = sqlx::query!(
            r#"
                SELECT sentiment AS "sentiment: Sentiment", COUNT(sentiment) as count
                FROM question_submission
                WHERE question_id = $1
                GROUP BY sentiment
                ORDER BY count DESC
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(sentiment_counts
            .into_iter()
            .filter(|s| s.sentiment.is_some() && s.count.is_some())
            .map(|s| SentimentCountResult {
                sentiment: s.sentiment.unwrap(),
                count: s.count.unwrap(),
            })
            .collect())
    }
}

#[ComplexObject]
impl QuestionSubmissionResult {
    async fn respondent(&self, ctx: &Context<'_>) -> Result<Option<RespondentResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        if let Some(respondent_id) = self.respondent_id.clone() {
            let respondent = sqlx::query_as!(
                Respondent,
                r#"
                    SELECT
                      id,
                      name,
                      email,
                      created_at,
                      updated_at
                    FROM respondent
                    WHERE id = $1
                "#,
                uuid::Uuid::parse_str(respondent_id.as_str()).unwrap(),
            )
            .fetch_one(&db_pool)
            .await?;

            Ok(Some(respondent.into()))
        } else {
            Ok(None)
        }
    }
}

impl From<Question> for QuestionResult {
    fn from(q: Question) -> Self {
        Self {
            id: q.id.into(),
            prompt: q.prompt,
            response_char_limit: q.response_char_limit,
            response_placeholder_text: q.response_placeholder_text,
            allow_anonymous_responses: q.allow_anonymous_responses,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }
    }
}

impl From<QuestionSubmission> for QuestionSubmissionResult {
    fn from(q: QuestionSubmission) -> Self {
        Self {
            id: q.id.into(),
            respondent_id: q.respondent_id.map(|id| id.into()),
            response: q.response,
            sentiment: q.sentiment,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }
    }
}

impl From<Respondent> for RespondentResult {
    fn from(r: Respondent) -> Self {
        Self {
            name: r.name,
            email: r.email,
        }
    }
}
