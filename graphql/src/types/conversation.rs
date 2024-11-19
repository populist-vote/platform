use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject, ID};
use auth::AccessTokenClaims;
use chrono::{DateTime, Utc};
use db::{models::conversation::Conversation, ArgumentPosition, UserWithProfile};
use jsonwebtoken::TokenData;
use uuid::Uuid;

use crate::{context::ApiContext, SessionData};

use super::UserResult;

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
enum StatementSort {
    Newest,
    MostVotes,
    MostAgree,
    Controversial,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ConversationResult {
    id: ID,
    topic: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct StatementResult {
    id: ID,
    conversation_id: ID,
    author_id: Option<ID>,
    content: String,
    created_at: DateTime<Utc>,
    vote_count: i64,
    agree_count: i64,
    disagree_count: i64,
    pass_count: i64,
}

#[derive(SimpleObject)]
struct ConversationStats {
    total_participants: i64,
    total_statements: i64,
    total_votes: i64,
    avg_votes_per_participant: f64,
}

#[derive(SimpleObject)]
struct TimeSeriesPoint {
    timestamp: DateTime<Utc>,
    count: i64,
}

#[derive(SimpleObject)]
struct ParticipationBucket {
    vote_count: i64,
    participant_count: i64,
    percentage_of_total: f64,
}

#[derive(SimpleObject)]
struct VoteDistributionBucket {
    vote_count: i64,        // Number of votes in this bucket (e.g., "5 votes")
    participant_count: i64, // How many participants cast this many votes
    percentage: f64,        // What percentage of total participants this represents
}

impl From<Conversation> for ConversationResult {
    fn from(conversation: Conversation) -> Self {
        Self {
            id: ID(conversation.id.to_string()),
            topic: conversation.topic,
            description: conversation.description,
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
        }
    }
}

#[ComplexObject]
impl ConversationResult {
    async fn statements(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: Option<StatementSort>,
    ) -> async_graphql::Result<Vec<StatementResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let order_by = match sort.unwrap_or(StatementSort::Newest) {
            StatementSort::Newest => "s.created_at DESC",
            StatementSort::MostVotes => "vote_count DESC, s.created_at DESC",
            StatementSort::MostAgree => "agree_count DESC, s.created_at DESC",
            StatementSort::Controversial => "(agree_count * disagree_count)::float / NULLIF(vote_count * vote_count, 0) DESC, s.created_at DESC",
        };

        let statements = sqlx::query!(
            r#"
            SELECT 
                s.id,
                s.conversation_id,
                s.author_id,
                s.content,
                s.created_at,
                COALESCE(COUNT(v.id), 0) as "vote_count!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'support'), 0) as "agree_count!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'oppose'), 0) as "disagree_count!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'neutral'), 0) as "pass_count!: i64"
            FROM statement s
            LEFT JOIN statement_vote v ON s.id = v.statement_id
            WHERE s.conversation_id = $1
            GROUP BY s.id
            ORDER BY 
            CASE WHEN $4 = 's.created_at DESC' THEN s.created_at END DESC,
            CASE WHEN $4 = 'vote_count DESC, s.created_at DESC' THEN COUNT(v.id) END DESC,
            CASE WHEN $4 = 'agree_count DESC, s.created_at DESC' THEN COUNT(*) FILTER (WHERE v.vote_type = 'support') END DESC,
            CASE WHEN $4 = '(agree_count * disagree_count)::float / NULLIF(vote_count * vote_count, 0) DESC, s.created_at DESC' 
                THEN (COUNT(*) FILTER (WHERE v.vote_type = 'oppose') * COUNT(*) FILTER (WHERE v.vote_type = 'oppose'))::float / 
                     NULLIF(COUNT(v.id) * COUNT(v.id), 0) 
            END DESC
            LIMIT $2 OFFSET $3
            "#,
            uuid::Uuid::parse_str(&self.id)?,
            limit as i64,
            offset as i64,
            order_by
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(statements
            .into_iter()
            .map(|row| StatementResult {
                id: row.id.into(),
                conversation_id: row.conversation_id.into(),
                author_id: row.author_id.map(|id| id.into()),
                content: row.content,
                created_at: row.created_at,
                vote_count: row.vote_count,
                agree_count: row.agree_count,
                disagree_count: row.disagree_count,
                pass_count: row.pass_count,
            })
            .collect())
    }

    async fn related_statements(
        &self,
        ctx: &Context<'_>,
        draft_content: String,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<StatementResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let limit = limit.unwrap_or(5);

        let statements = sqlx::query!(
            r#"
            WITH statement_search AS (
                SELECT 
                    s.id,
                    s.conversation_id,
                    s.author_id,
                    s.content,
                    s.created_at,
                    COALESCE(COUNT(v.id), 0) as "vote_count!: i64",
                    COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'support'), 0) as "agree_count!: i64",
                    COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'oppose'), 0) as "disagree_count!: i64",
                    COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'neutral'), 0) as "pass_count!: i64",
                    (
                        -- Combine different similarity metrics
                        0.4 * similarity(lower(s.content), lower($1)) + -- Exact similarity
                        0.4 * ts_rank_cd(
                            setweight(to_tsvector('english', s.content), 'A'),
                            to_tsquery('english', regexp_replace(
                                trim(regexp_replace($1, '[^a-zA-Z0-9\s]', ' ', 'g')),
                                '\s+',
                                ' | ',
                                'g'
                            )),
                            32
                        ) +
                        0.2 * (1 - word_similarity(lower(s.content), lower($1))) -- Word-level similarity
                    ) * (1 + ln(GREATEST(COUNT(v.id), 1))) as similarity_score -- Engagement boost
                FROM statement s
                LEFT JOIN statement_vote v ON s.id = v.statement_id
                WHERE 
                    s.conversation_id = $2 AND
                    (
                        -- Multiple matching conditions
                        similarity(lower(s.content), lower($1)) > 0.1 OR -- Basic trigram similarity
                        s.content ILIKE '%' || $1 || '%' OR -- Contains the search term
                        to_tsvector('english', s.content) @@ to_tsquery('english', 
                            regexp_replace(
                                trim(regexp_replace($1, '[^a-zA-Z0-9\s]', ' ', 'g')),
                                '\s+',
                                ' | ',
                                'g'
                            )
                        ) -- Full text search with OR instead of AND
                    )
                GROUP BY s.id
            )
            SELECT *
            FROM statement_search
            WHERE similarity_score > 0.1
            ORDER BY similarity_score DESC
            LIMIT $3;
            "#,
            draft_content,
            uuid::Uuid::parse_str(&self.id)?,
            limit as i64
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(statements
            .into_iter()
            .map(|row| StatementResult {
                id: row.id.into(),
                conversation_id: row.conversation_id.into(),
                author_id: row.author_id.map(|id| id.into()),
                content: row.content,
                created_at: row.created_at,
                vote_count: row.vote_count,
                agree_count: row.agree_count,
                disagree_count: row.disagree_count,
                pass_count: row.pass_count,
            })
            .collect())
    }

    async fn stats(&self, ctx: &Context<'_>) -> async_graphql::Result<ConversationStats> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let stats = sqlx::query!(
            r#"
            WITH participant_stats AS (
                SELECT 
                    COUNT(DISTINCT COALESCE(user_id, session_id)) as unique_participants,
                    COUNT(*) as total_votes
                FROM statement_vote v
                JOIN statement s ON v.statement_id = s.id
                WHERE s.conversation_id = $1
            ),
            statement_stats AS (
                SELECT COUNT(*) as total_statements
                FROM statement
                WHERE conversation_id = $1
            )
            SELECT 
                p.unique_participants as "total_participants!",
                s.total_statements as "total_statements!",
                p.total_votes as "total_votes!",
                CASE 
                    WHEN p.unique_participants > 0 
                    THEN p.total_votes::float / p.unique_participants::float 
                    ELSE 0 
                END as "avg_votes_per_participant!"
            FROM participant_stats p, statement_stats s
            "#,
            uuid::Uuid::parse_str(&self.id)?
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(ConversationStats {
            total_participants: stats.total_participants,
            total_statements: stats.total_statements,
            total_votes: stats.total_votes,
            avg_votes_per_participant: stats.avg_votes_per_participant,
        })
    }

    async fn statements_over_time(
        &self,
        ctx: &Context<'_>,
        interval: Option<String>,
    ) -> async_graphql::Result<Vec<TimeSeriesPoint>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let interval = interval.unwrap_or("1 day".to_string());
        let interval_unit = match interval.split_whitespace().nth(1) {
            Some(unit) => unit,
            None => return Err(async_graphql::Error::new("Invalid interval format")),
        };

        let points = sqlx::query!(
            r#"
            WITH RECURSIVE timeline AS (
                SELECT 
                    date_trunc($2::text, MIN(created_at)) as time_bucket,
                    date_trunc($2::text, MAX(created_at)) as max_time
                FROM statement
                WHERE conversation_id = $1
                UNION ALL
                SELECT 
                    time_bucket + ($3::text::interval),
                    max_time
                FROM timeline
                WHERE time_bucket < max_time
            )
            SELECT 
                t.time_bucket as "timestamp!",
                COUNT(s.id) as "count!"
            FROM timeline t
            LEFT JOIN statement s ON 
                date_trunc($2::text, s.created_at) <= t.time_bucket AND
                s.conversation_id = $1
            GROUP BY t.time_bucket
            ORDER BY t.time_bucket
            "#,
            uuid::Uuid::parse_str(&self.id)?,
            interval_unit,
            interval
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(points
            .into_iter()
            .map(|p| TimeSeriesPoint {
                timestamp: p.timestamp,
                count: p.count,
            })
            .collect())
    }

    async fn votes_over_time(
        &self,
        ctx: &Context<'_>,
        interval: Option<String>,
    ) -> async_graphql::Result<Vec<TimeSeriesPoint>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let interval = interval.unwrap_or("1 day".to_string());
        let interval_unit = match interval.split_whitespace().nth(1) {
            Some(unit) => unit,
            None => return Err(async_graphql::Error::new("Invalid interval format")),
        };

        let points = sqlx::query!(
            r#"
            WITH RECURSIVE timeline AS (
                SELECT 
                    date_trunc($2::text, MIN(v.created_at)) as time_bucket,
                    date_trunc($2::text, MAX(v.created_at)) as max_time
                FROM statement_vote v
                JOIN statement s ON v.statement_id = s.id
                WHERE s.conversation_id = $1
                UNION ALL
                SELECT 
                    time_bucket + ($3::text::interval),
                    max_time
                FROM timeline
                WHERE time_bucket < max_time
            )
            SELECT 
                t.time_bucket as "timestamp!",
                COUNT(v.id) as "count!"
            FROM timeline t
            LEFT JOIN statement s ON s.conversation_id = $1
            LEFT JOIN statement_vote v ON 
                v.statement_id = s.id AND
                date_trunc($2::text, v.created_at) <= t.time_bucket
            GROUP BY t.time_bucket
            ORDER BY t.time_bucket
            "#,
            uuid::Uuid::parse_str(&self.id)?,
            interval_unit,
            interval
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(points
            .into_iter()
            .map(|p| TimeSeriesPoint {
                timestamp: p.timestamp,
                count: p.count,
            })
            .collect())
    }

    /// Counts unique participants who voted in each time bucket
    async fn participation_over_time(
        &self,
        ctx: &Context<'_>,
        interval: Option<String>, // e.g., '1 hour', '1 day'
    ) -> async_graphql::Result<Vec<TimeSeriesPoint>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let interval = interval.unwrap_or("1 day".to_string());
        let interval_unit = match interval.split_whitespace().nth(1) {
            Some(unit) => unit,
            None => return Err(async_graphql::Error::new("Invalid interval format")),
        };

        let points = sqlx::query!(
            r#"
            WITH RECURSIVE timeline AS (
                SELECT 
                    date_trunc($2, MIN(v.created_at)) as time_bucket,
                    date_trunc($2, MAX(v.created_at)) as max_time
                FROM statement_vote v
                JOIN statement s ON v.statement_id = s.id
                WHERE s.conversation_id = $1
                UNION ALL
                SELECT 
                    time_bucket + ($3::text::interval),
                    max_time
                FROM timeline
                WHERE time_bucket < max_time
            ),
            participants AS (
                SELECT DISTINCT
                    COALESCE(user_id, session_id) as participant_id,
                    date_trunc($2::text, v.created_at) as time_bucket
                FROM statement_vote v
                JOIN statement s ON v.statement_id = s.id
                WHERE s.conversation_id = $1
            )
            SELECT 
                t.time_bucket as "timestamp!",
                COUNT(DISTINCT participant_id) as "count!"
            FROM timeline t
            LEFT JOIN participants p ON p.time_bucket <= t.time_bucket
            GROUP BY t.time_bucket
            ORDER BY t.time_bucket
            "#,
            uuid::Uuid::parse_str(&self.id)?,
            interval_unit,
            interval
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(points
            .into_iter()
            .map(|p| TimeSeriesPoint {
                timestamp: p.timestamp,
                count: p.count,
            })
            .collect())
    }

    /// Returns distribution of voting activity across participants.
    /// Shows how many participants cast X number of votes.
    async fn vote_distribution(
        &self,
        ctx: &Context<'_>,
        bucket_size: Option<i32>, // Optional parameter to group votes into ranges
    ) -> async_graphql::Result<Vec<VoteDistributionBucket>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let bucket_size = bucket_size.unwrap_or(1); // Default to exact counts

        let distribution = sqlx::query!(
            r#"
            WITH participant_vote_counts AS (
                -- First, count how many votes each participant cast
                SELECT 
                    session_id,
                    COUNT(*) as vote_count
                FROM statement_vote v
                JOIN statement s ON v.statement_id = s.id
                WHERE s.conversation_id = $1
                GROUP BY session_id
            ),
            bucketed_counts AS (
                -- Then bucket these counts and count participants per bucket
                SELECT
                    -- Round down to nearest bucket_size
                    (vote_count / $2 * $2) as votes_cast,
                    COUNT(*) as participant_count
                FROM participant_vote_counts
                GROUP BY (vote_count / $2 * $2)
            ),
            total_participants AS (
                -- Get total participant count for percentage calculation
                SELECT COUNT(*) as total
                FROM participant_vote_counts
            )
            -- Finally, calculate percentages and return results
            SELECT 
                votes_cast as "votes_cast!",
                participant_count as "participant_count!",
                (participant_count::float / NULLIF(total, 0) * 100) as "percentage!"
            FROM bucketed_counts, total_participants
            ORDER BY votes_cast
            "#,
            uuid::Uuid::parse_str(&self.id)?,
            bucket_size as i64
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(distribution
            .into_iter()
            .map(|row| VoteDistributionBucket {
                vote_count: row.votes_cast,
                participant_count: row.participant_count,
                percentage: row.percentage,
            })
            .collect())
    }
}

#[ComplexObject]
impl StatementResult {
    async fn author(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<UserResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        match &self.author_id {
            Some(author_id) => {
                let record = sqlx::query_as!(
                    UserWithProfile,
                    r#"
                    SELECT u.id, u.username, u.email, first_name, last_name, profile_picture_url FROM user_profile up
                    JOIN populist_user u ON up.user_id = u.id WHERE u.id = $1
                "#,
                    uuid::Uuid::parse_str(&author_id)?,
                )
                .fetch_optional(&db_pool)
                .await?;

                match record {
                    Some(user) => Ok(Some(user.into())),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    async fn vote_by_user_or_session(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
    ) -> Result<Option<ArgumentPosition>> {
        let session_data = ctx.data::<SessionData>()?.clone();
        let session_id = uuid::Uuid::parse_str(&session_data.session_id.to_string())?;

        let user_id = match user_id {
            Some(user_id) => Some(
                Uuid::parse_str(&user_id.to_string()).map_err(|_| Error::new("Invalid user ID"))?,
            ),
            None => match ctx.data::<TokenData<AccessTokenClaims>>() {
                Ok(token_data) => Some(token_data.claims.sub),
                Err(_) => None,
            },
        };

        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let vote = sqlx::query!(
            r#"
            SELECT vote_type as "vote_type: ArgumentPosition"
            FROM statement_vote
            WHERE statement_id = $1 AND (
                user_id = $2 OR session_id = $3
            )
            "#,
            Uuid::parse_str(&self.id.to_string())?,
            user_id,
            session_id
        )
        .fetch_optional(&db_pool)
        .await?;

        let vote = vote.map(|v| v.vote_type);

        Ok(vote)
    }
}
