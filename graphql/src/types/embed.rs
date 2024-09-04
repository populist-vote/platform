use crate::is_admin;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{candidate_guide::CandidateGuide, election},
    DateTime, Embed, EmbedType, UserWithProfile,
};
use serde_json::Value as JSON;

use crate::context::ApiContext;

use super::{
    BillResult, CandidateGuideRaceResult, CandidateGuideResult, ElectionResult, Error,
    PoliticianResult, PollResult, QuestionResult, RaceResult, UserResult,
};

#[derive(SimpleObject, Clone, Debug)]
#[graphql(complex)]
pub struct EmbedResult {
    pub id: ID,
    pub organization_id: ID,
    pub name: String,
    pub description: Option<String>,
    pub embed_type: EmbedType,
    pub attributes: JSON,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub created_by_id: ID,
    pub updated_by_id: ID,
}

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
pub struct EmbedOriginResult {
    pub url: String,
    pub last_ping_at: DateTime,
}

#[ComplexObject]
impl EmbedResult {
    async fn created_by(&self, ctx: &Context<'_>) -> Result<UserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            UserWithProfile,
            r#"
            SELECT u.id, u.username, u.email, first_name, last_name, profile_picture_url FROM user_profile up
            JOIN populist_user u ON up.user_id = u.id WHERE u.id = $1
        "#,
            uuid::Uuid::parse_str(&self.created_by_id.as_str()).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        match record {
            Some(user) => Ok(user.into()),
            None => Err(Error::UserNotFound.into()),
        }
    }

    async fn updated_by(&self, ctx: &Context<'_>) -> Result<UserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            UserWithProfile,
            r#"
            SELECT u.id, u.username, u.email, first_name, last_name, profile_picture_url FROM user_profile up
            JOIN populist_user u ON up.user_id = u.id WHERE u.id = $1
        "#,
            uuid::Uuid::parse_str(&self.updated_by_id.as_str()).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        match record {
            Some(user) => Ok(user.into()),
            None => Err(Error::UserNotFound.into()),
        }
    }

    async fn origins(&self, ctx: &Context<'_>) -> Result<Vec<EmbedOriginResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            EmbedOriginResult,
            r#"
            SELECT url, last_ping_at FROM embed_origin WHERE embed_id = $1
            AND url NOT LIKE '%localhost:3030%'
            AND url NOT LIKE '%staging.populist.us%'
            AND url NOT LIKE '%populist.us%'
        "#,
            uuid::Uuid::parse_str(&self.id.as_str()).unwrap(),
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(records)
    }

    async fn bill(&self, ctx: &Context<'_>) -> Result<Option<BillResult>> {
        let bill_id = self.attributes["billId"].as_str();
        if let Some(bill_id) = bill_id {
            let bill_id = uuid::Uuid::parse_str(bill_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Bill::find_by_id(&db_pool, bill_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    async fn bills(&self, ctx: &Context<'_>) -> Result<Option<Vec<BillResult>>> {
        let bill_ids = self.attributes["billIds"].as_array();
        if let Some(bill_ids) = bill_ids {
            let bill_ids = bill_ids
                .iter()
                .filter_map(|id| id.as_str().map(|id| uuid::Uuid::parse_str(id).ok()))
                .collect::<Option<Vec<_>>>()
                .ok_or(Error::BadInput {
                    field: "bill_ids".to_string(),
                    message: "Bill ids we malformed or missing".to_string(),
                })?;

            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let records = db::Bill::find_by_ids(&db_pool, bill_ids).await?;
            Ok(Some(records.into_iter().map(|r| r.into()).collect()))
        } else {
            Ok(None)
        }
    }

    async fn politician(&self, ctx: &Context<'_>) -> Result<Option<PoliticianResult>> {
        let politician_id = self.attributes["politicianId"].as_str();
        if let Some(politician_id) = politician_id {
            let politician_id = uuid::Uuid::parse_str(politician_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Politician::find_by_id(&db_pool, politician_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    async fn race(&self, ctx: &Context<'_>) -> Result<Option<RaceResult>> {
        let race_id = self.attributes["raceId"].as_str();
        if let Some(race_id) = race_id {
            let race_id = uuid::Uuid::parse_str(race_id)?;
            let race = ctx
                .data::<ApiContext>()?
                .loaders
                .race_loader
                .load_one(race_id)
                .await?;
            Ok(race.map(|r| r.into()))
        } else {
            Ok(None)
        }
    }

    async fn candidate_guide_race(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<CandidateGuideRaceResult>> {
        let race_id = self.attributes["raceId"].as_str();
        let candidate_guide_id = self.attributes["candidateGuideId"].as_str();

        if let (Some(race_id), Some(candidate_guide_id)) = (race_id, candidate_guide_id) {
            let race_id = uuid::Uuid::parse_str(race_id)?;
            let race = ctx
                .data::<ApiContext>()?
                .loaders
                .race_loader
                .load_one(race_id)
                .await?;
            let race_result = race.map(|r| r.into());

            let candidate_guide_id = uuid::Uuid::parse_str(candidate_guide_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let cgr = sqlx::query!(
                r#"
                SELECT were_candidates_emailed, created_at, updated_at FROM candidate_guide_races
                WHERE race_id = $1 AND candidate_guide_id = $2
                "#,
                race_id,
                candidate_guide_id
            )
            .fetch_optional(&db_pool)
            .await?;

            if let (Some(race), Some(cgr)) = (race_result, cgr) {
                let result = CandidateGuideRaceResult {
                    race,
                    were_candidates_emailed: cgr.were_candidates_emailed.unwrap_or(false),
                    created_at: cgr.created_at,
                    updated_at: cgr.updated_at,
                };
                Ok(Some(result))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn question(&self, ctx: &Context<'_>) -> Result<Option<QuestionResult>> {
        let question_id = self.attributes["questionId"].as_str();
        if let Some(question_id) = question_id {
            let question_id = uuid::Uuid::parse_str(question_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Question::find_by_id(&db_pool, question_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    async fn poll(&self, ctx: &Context<'_>) -> Result<Option<PollResult>> {
        let poll_id = self.attributes["pollId"].as_str();
        if let Some(poll_id) = poll_id {
            let poll_id = uuid::Uuid::parse_str(poll_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Poll::find_by_id(&db_pool, poll_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    async fn candidate_guide(&self, ctx: &Context<'_>) -> Result<Option<CandidateGuideResult>> {
        let candidate_guide_id = self.attributes["candidateGuideId"].as_str();
        if let Some(candidate_guide_id) = candidate_guide_id {
            let candidate_guide_id = uuid::Uuid::parse_str(candidate_guide_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = CandidateGuide::find_by_id(&db_pool, candidate_guide_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    async fn election(&self, ctx: &Context<'_>) -> Result<Option<ElectionResult>> {
        let election_id = self.attributes["electionId"].as_str();
        if let Some(election_id) = election_id {
            let election_id = uuid::Uuid::parse_str(election_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record = db::Election::find_by_id(&db_pool, election_id).await?;
            Ok(Some(record.into()))
        } else {
            Ok(None)
        }
    }

    /// Each candidate guide embed is associated with a single race. This field returns the
    /// the number of questions submitted by candidates in this race, divided by the number
    /// of questions in a candidate guide
    async fn candidate_guide_submission_count_by_race(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<i64>> {
        let candidate_guide_id = self.attributes["candidateGuideId"].as_str();
        let race_id = self.attributes["raceId"].as_str();
        if let (Some(candidate_guide_id), Some(race_id)) = (candidate_guide_id, race_id) {
            let candidate_guide_id = uuid::Uuid::parse_str(candidate_guide_id)?;
            let race_id = uuid::Uuid::parse_str(race_id)?;
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let result = sqlx::query!(
                r#"
                SELECT
                    COUNT(DISTINCT qs.id) AS total_submissions,
                    COUNT(DISTINCT cgq.question_id) AS total_questions
                FROM
                    candidate_guide_questions cgq
                    JOIN question_submission qs ON cgq.question_id = qs.question_id
                    JOIN candidate_guide_races cgr ON cgr.candidate_guide_id = $1
                    JOIN race_candidates rc ON cgr.race_id = rc.race_id
                WHERE
                    cgq.candidate_guide_id = $1
                    AND cgr.race_id = $2
                    AND rc.candidate_id = qs.candidate_id
                    AND qs.response IS NOT NULL
                    AND qs.response != ''
            "#,
                candidate_guide_id,
                race_id
            )
            .fetch_one(&db_pool)
            .await?;
            let total_submissions = result.total_submissions.unwrap_or(0);
            let total_questions = result.total_questions.unwrap_or(0);
            let count = if total_questions > 0 {
                total_submissions as i64 / total_questions as i64
            } else {
                0
            };
            Ok(Some(count))
        } else {
            Ok(None)
        }
    }
}

impl From<Embed> for EmbedResult {
    fn from(embed: Embed) -> Self {
        Self {
            id: embed.id.into(),
            organization_id: embed.organization_id.into(),
            name: embed.name,
            description: embed.description,
            embed_type: embed.embed_type,
            attributes: embed.attributes,
            created_at: embed.created_at,
            updated_at: embed.updated_at,
            created_by_id: embed.created_by.into(),
            updated_by_id: embed.updated_by.into(),
        }
    }
}
