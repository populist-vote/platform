use crate::context::ApiContext;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    DateTime, Embed, EmbedType, IssueTag, Question, QuestionSubmission, Respondent, Sentiment,
};

use super::{EmbedResult, IssueTagResult, PoliticianResult, SubmissionCountByDateResult};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct QuestionResult {
    id: ID,
    prompt: String,
    translations: Option<serde_json::Value>,
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
    question_id: ID,
    respondent_id: Option<ID>,
    candidate_id: Option<ID>,
    response: String,
    editorial: Option<String>,
    translations: Option<serde_json::Value>,
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
    async fn issue_tags(&self, ctx: &Context<'_>) -> Result<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let issue_tags = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN question_issue_tags
                ON question_issue_tags.issue_tag_id = it.id
                WHERE question_issue_tags.question_id = $1
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(issue_tags.into_iter().map(|it| it.into()).collect())
    }

    async fn submissions(&self, ctx: &Context<'_>) -> Result<Vec<QuestionSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submissions = sqlx::query_as!(
            QuestionSubmission,
            r#"
                SELECT
                  id,
                  question_id,
                  respondent_id,
                  candidate_id,
                  response,
                  editorial,
                  translations,
                  sentiment AS "sentiment: Sentiment",
                  is_locked,
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

    async fn submissions_by_race(
        &self,
        ctx: &Context<'_>,
        race_id: ID,
    ) -> Result<Vec<QuestionSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submissions = sqlx::query_as!(
            QuestionSubmission,
            r#"
                SELECT
                  qs.id,
                  qs.question_id,
                  qs.respondent_id,
                  qs.candidate_id,
                  qs.response,
                  qs.editorial,
                  qs.translations,
                  qs.sentiment AS "sentiment: Sentiment",
                  qs.is_locked,
                  qs.created_at,
                  qs.updated_at
                FROM question_submission qs
                JOIN race_candidates rc
                ON qs.candidate_id = rc.candidate_id
                WHERE qs.question_id = $1
                AND rc.race_id = $2
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
            uuid::Uuid::parse_str(race_id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(submissions.into_iter().map(|s| s.into()).collect())
    }

    async fn submissions_by_candidate_id(
        &self,
        ctx: &Context<'_>,
        candidate_id: ID,
    ) -> Result<Vec<QuestionSubmissionResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let submissions = sqlx::query_as!(
            QuestionSubmission,
            r#"
                SELECT
                  qs.id,
                  qs.question_id,
                  qs.respondent_id,
                  qs.candidate_id,
                  qs.response,
                  qs.editorial,
                  qs.translations,
                  qs.sentiment AS "sentiment: Sentiment",
                  qs.is_locked,
                  qs.created_at,
                  qs.updated_at
                FROM question_submission qs
                WHERE qs.question_id = $1
                AND qs.candidate_id = $2
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap(),
            uuid::Uuid::parse_str(candidate_id.as_str()).unwrap(),
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

    async fn politician(&self, ctx: &Context<'_>) -> Result<Option<PoliticianResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        if let Some(candidate_id) = self.candidate_id.clone() {
            let politician = db::Politician::find_by_id(
                &db_pool,
                uuid::Uuid::parse_str(candidate_id.as_str()).unwrap(),
            )
            .await?;

            Ok(Some(politician.into()))
        } else {
            Ok(None)
        }
    }

    async fn question(&self, ctx: &Context<'_>) -> Result<QuestionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let question = db::Question::find_by_id(
            &db_pool,
            uuid::Uuid::parse_str(self.question_id.as_str()).unwrap(),
        )
        .await?;

        Ok(question.into())
    }

    /// Returns the candidate guide embed associated with the question submission.
    async fn candidate_guide_embed(&self, ctx: &Context<'_>) -> Result<EmbedResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let embed = sqlx::query_as!(
            Embed,
            r#"
                SELECT e.id, e.organization_id, e.name, e.description, e.attributes, e.embed_type AS "embed_type:EmbedType", e.created_at, e.created_by, e.updated_at, e.updated_by
                FROM question_submission qs
                JOIN question q ON qs.question_id = q.id
                JOIN candidate_guide_questions cgq ON q.id = cgq.question_id
                JOIN candidate_guide cg ON cgq.candidate_guide_id = cg.id
                JOIN embed e ON (e.attributes->>'candidateGuideId')::uuid = cg.id
                JOIN race_candidates rc ON rc.candidate_id = qs.candidate_id
                    AND (e.attributes ->> 'raceId')::uuid = rc.race_id
                WHERE qs.id = $1
                LIMIT 1;
        "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap()
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(embed.into())
    }
}

impl From<Question> for QuestionResult {
    fn from(q: Question) -> Self {
        Self {
            id: q.id.into(),
            prompt: q.prompt,
            translations: q.translations,
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
            question_id: q.question_id.into(),
            respondent_id: q.respondent_id.map(|id| id.into()),
            candidate_id: q.candidate_id.map(|id| id.into()),
            response: q.response,
            editorial: q.editorial,
            translations: q.translations,
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
