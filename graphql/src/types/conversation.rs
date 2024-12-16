use std::collections::{HashMap, HashSet};

use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject, ID};
use async_openai::{
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role},
    Client,
};
use auth::AccessTokenClaims;
use chrono::{DateTime, Utc};
use db::{
    models::conversation::{Conversation, StatementView, StatementVote},
    ArgumentPosition, StatementModerationStatus, UserWithProfile,
};
use jsonwebtoken::TokenData;

use ndarray::{Array2, ArrayView1, Axis};

use rand::Rng;
use sqlx::PgPool;
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

#[derive(async_graphql::InputObject, Clone, Eq, PartialEq, Default)]
struct StatementFilter {
    moderation_statuses: Option<Vec<StatementModerationStatus>>,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ConversationResult {
    pub id: ID,
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
    total_votes: i64,
    support_votes: i64,
    oppose_votes: i64,
    neutral_votes: i64,
    moderation_status: StatementModerationStatus,
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

#[derive(SimpleObject, Clone)]
struct OpinionScore {
    id: String,
    content: String,
    score: f64,
    total_votes: i32,
    support_votes: i32,
    oppose_votes: i32,
    neutral_votes: i32,
    total_views: i32,
    non_voting_views: i32,
}

#[derive(SimpleObject)]
struct OpinionAnalysis {
    overview: Option<String>,
    consensus_opinions: Vec<OpinionScore>,
    divisive_opinions: Vec<OpinionScore>,
}

#[derive(SimpleObject, Debug)]
struct OpinionGroup {
    id: ID,
    users: Vec<ID>, // Using String to represent UUIDs
    characteristic_votes: Vec<CharacteristicVote>,
    summary: String,
}

#[derive(SimpleObject, Debug)]
#[graphql(complex)]
struct CharacteristicVote {
    statement_id: ID,
    mean_sentiment: f64,
    consensus_level: f64,
    significance_level: f64,
}

#[ComplexObject]
impl CharacteristicVote {
    async fn statement(&self, ctx: &Context<'_>) -> async_graphql::Result<StatementResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let statement = sqlx::query!(
            r#"
            SELECT 
                s.id,
                s.conversation_id,
                s.author_id,
                s.content,
                s.moderation_status as "moderation_status: StatementModerationStatus",
                s.created_at,
                COALESCE(COUNT(v.id), 0) as "vote_count!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'support'), 0) as "support_votes!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'oppose'), 0) as "oppose_votes!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'neutral'), 0) as "neutral_votes!: i64"
            FROM statement s
            LEFT JOIN statement_vote v ON s.id = v.statement_id
            WHERE s.id = $1
            GROUP BY s.id
            "#,
            Uuid::parse_str(&self.statement_id.to_string())?
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(StatementResult {
            id: statement.id.into(),
            conversation_id: statement.conversation_id.into(),
            author_id: statement.author_id.map(|id| id.into()),
            content: statement.content,
            moderation_status: statement.moderation_status.into(),
            created_at: statement.created_at,
            total_votes: statement.vote_count,
            support_votes: statement.support_votes,
            oppose_votes: statement.oppose_votes,
            neutral_votes: statement.neutral_votes,
        })
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
        filter: Option<StatementFilter>,
    ) -> async_graphql::Result<Vec<StatementResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        let moderation_statuses: Option<Vec<StatementModerationStatus>> =
            filter.unwrap_or_default().moderation_statuses;

        let order_by: &str = match sort.unwrap_or(StatementSort::Newest) {
            StatementSort::Newest => "s.created_at DESC",
            StatementSort::MostVotes => "total_votes DESC, s.created_at DESC",
            StatementSort::MostAgree => "support_votes DESC, s.created_at DESC",
            StatementSort::Controversial => "(support_votes * oppose_votes)::float / NULLIF(total_votes * total_votes, 0) DESC, s.created_at DESC",
        };

        let statements = sqlx::query!(
            r#"
            SELECT 
                s.id,
                s.conversation_id,
                s.author_id,
                s.content,
                s.moderation_status as "moderation_status: StatementModerationStatus",
                s.created_at,
                COALESCE(COUNT(v.id), 0) as "total_votes!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'support'), 0) as "support_votes!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'oppose'), 0) as "oppose_votes!: i64",
                COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'neutral'), 0) as "neutral_votes!: i64"
            FROM statement s
            LEFT JOIN statement_vote v ON s.id = v.statement_id
            WHERE s.conversation_id = $1
            AND ($5::_statement_moderation_status IS NULL OR 
                 s.moderation_status = ANY($5::_statement_moderation_status))
            GROUP BY s.id
            ORDER BY 
            CASE WHEN $4 = 's.created_at DESC' THEN s.created_at END DESC,
            CASE WHEN $4 = 'total_votes DESC, s.created_at DESC' THEN COUNT(v.id) END DESC,
            CASE WHEN $4 = 'support_votes DESC, s.created_at DESC' THEN COUNT(*) FILTER (WHERE v.vote_type = 'support') END DESC,
            CASE WHEN $4 = '(support_votes * oppose_votes)::float / NULLIF(total_votes * total_votes, 0) DESC, s.created_at DESC' 
                THEN (COUNT(*) FILTER (WHERE v.vote_type = 'oppose') * COUNT(*) FILTER (WHERE v.vote_type = 'oppose'))::float / 
                     NULLIF(COUNT(v.id) * COUNT(v.id), 0) 
            END DESC
            LIMIT $2 OFFSET $3
            "#,
            uuid::Uuid::parse_str(&self.id)?,
            limit as i64,
            offset as i64,
            order_by,
            moderation_statuses as _,
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
                moderation_status: row.moderation_status.into(),
                created_at: row.created_at,
                total_votes: row.total_votes,
                support_votes: row.support_votes,
                oppose_votes: row.oppose_votes,
                neutral_votes: row.neutral_votes,
            })
            .collect())
    }

    async fn related_statements(
        &self,
        ctx: &Context<'_>,
        draft_content: String,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Option<StatementFilter>,
    ) -> async_graphql::Result<Vec<StatementResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let limit = limit.unwrap_or(5);
        let offset = offset.unwrap_or(0);
        let moderation_statuses: Option<Vec<StatementModerationStatus>> =
            filter.unwrap_or_default().moderation_statuses;

        let statements = sqlx::query!(
            r#"
            WITH statement_search AS (
                SELECT 
                    s.id,
                    s.conversation_id,
                    s.author_id,
                    s.content,
                    s.moderation_status as "moderation_status: StatementModerationStatus",
                    s.created_at,
                    COALESCE(COUNT(v.id), 0) as "vote_count!: i64",
                    COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'support'), 0) as "support_votes!: i64",
                    COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'oppose'), 0) as "oppose_votes!: i64",
                    COALESCE(COUNT(*) FILTER (WHERE v.vote_type = 'neutral'), 0) as "neutral_votes!: i64",
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
                    ($4::_statement_moderation_status IS NULL OR 
                     s.moderation_status = ANY($4::_statement_moderation_status)) AND
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
            LIMIT $3
            OFFSET $5;
            "#,
            draft_content,
            uuid::Uuid::parse_str(&self.id)?,
            limit as i64,
            moderation_statuses as _,
            offset as i64,
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
                moderation_status: row.moderation_status.into(),
                created_at: row.created_at,
                total_votes: row.vote_count,
                support_votes: row.support_votes,
                oppose_votes: row.oppose_votes,
                neutral_votes: row.neutral_votes,
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

    async fn opinion_analysis(&self, ctx: &Context<'_>, limit: i32) -> Result<OpinionAnalysis> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let statements_with_votes = fetch_statements_with_votes(&db_pool, &self.id).await?;

        // Process statements once to get both consensus and divisive opinions
        let mut consensus_statements: Vec<(StatementWithMeta, f64)> = Vec::new();
        let mut divisive_statements: Vec<(StatementWithMeta, f64)> = Vec::new();

        for statement in statements_with_votes
            .into_iter()
            .filter(|s| !s.votes.is_empty())
        {
            let vote_counts = count_votes(&statement.votes);
            let total_votes = statement.votes.len() as f64;

            // Only add to divisive if there's actually a mix of votes
            let has_vote_variety = vote_counts.values().filter(|&&count| count > 0).count() > 1;

            let consensus_score = calculate_consensus_score(&vote_counts, total_votes);
            consensus_statements.push((statement.clone(), consensus_score));

            if has_vote_variety {
                let divisiveness_score = calculate_divisiveness_score(&vote_counts, total_votes);
                divisive_statements.push((statement, divisiveness_score));
            }
        }

        // Sort and truncate both lists
        consensus_statements.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        divisive_statements.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let consensus_opinions: Vec<OpinionScore> = consensus_statements
            .into_iter()
            .take(limit as usize)
            .map(|(statement, score)| {
                let counts = count_votes(&statement.votes);
                // Get unique voting sessions
                let voting_sessions: HashSet<_> = statement
                    .votes
                    .iter()
                    .filter_map(|v| v.session_id.as_ref())
                    .collect();

                // Get unique viewing sessions
                let viewing_sessions: HashSet<_> =
                    statement.views.iter().map(|v| &v.session_id).collect();

                // Calculate non-voting views
                let non_voting_views = viewing_sessions.difference(&voting_sessions).count();
                OpinionScore {
                    id: statement.id.to_string(),
                    content: statement.content,
                    score,
                    total_votes: statement.votes.len() as i32,
                    support_votes: counts.get(&ArgumentPosition::Support).copied().unwrap_or(0),
                    oppose_votes: counts.get(&ArgumentPosition::Oppose).copied().unwrap_or(0),
                    neutral_votes: counts.get(&ArgumentPosition::Neutral).copied().unwrap_or(0),
                    total_views: viewing_sessions.len() as i32,
                    non_voting_views: non_voting_views as i32,
                }
            })
            .collect();

        let divisive_opinions: Vec<OpinionScore> = divisive_statements
            .into_iter()
            .take(limit as usize)
            .map(|(statement, score)| {
                let counts = count_votes(&statement.votes);
                // Get unique voting sessions
                let voting_sessions: HashSet<_> = statement
                    .votes
                    .iter()
                    .filter_map(|v| v.session_id.as_ref())
                    .collect();

                // Get unique viewing sessions
                let viewing_sessions: HashSet<_> =
                    statement.views.iter().map(|v| &v.session_id).collect();

                // Calculate non-voting views
                let non_voting_views = viewing_sessions.difference(&voting_sessions).count();
                OpinionScore {
                    id: statement.id.to_string(),
                    content: statement.content,
                    score,
                    total_votes: statement.votes.len() as i32,
                    support_votes: counts.get(&ArgumentPosition::Support).copied().unwrap_or(0),
                    oppose_votes: counts.get(&ArgumentPosition::Oppose).copied().unwrap_or(0),
                    neutral_votes: counts.get(&ArgumentPosition::Neutral).copied().unwrap_or(0),
                    total_views: viewing_sessions.len() as i32,
                    non_voting_views: non_voting_views as i32,
                }
            })
            .collect();

        let overview =
            match generate_opinion_summary(consensus_opinions.clone(), divisive_opinions.clone())
                .await
            {
                Ok(summary) => Some(summary),
                Err(_) => None,
            };

        Ok(OpinionAnalysis {
            overview,
            consensus_opinions,
            divisive_opinions,
        })
    }

    async fn opinion_groups(&self, ctx: &Context<'_>) -> Result<Vec<OpinionGroup>, Error> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        // Fetch all votes
        let votes = sqlx::query_as!(
            StatementVote,
            r#"
            SELECT v.id, statement_id, s.content, user_id, session_id, vote_type AS "vote_type: ArgumentPosition", v.created_at, v.updated_at
            FROM statement_vote v
            JOIN statement s ON v.statement_id = s.id
            WHERE s.conversation_id = $1
            "#,
            uuid::Uuid::parse_str(&self.id)?
        )
        .fetch_all(&db_pool)
        .await?;

        // Convert votes to numerical matrix
        let (matrix, voter_ids, statement_ids) = prepare_voting_matrix(&votes);

        // Determine optimal number of clusters
        let optimal_k = calculate_optimal_clusters(&matrix);

        // Perform k-means clustering with optimal k
        let groups = kmeans(&matrix, optimal_k, 100);

        println!("K-means output groups:");
        for (i, group) in groups.iter().enumerate() {
            println!("Group {}: {} members", i, group.len());
        }

        // Analyze groups
        let mut opinion_groups = Vec::new();
        for (group_id, group_indices) in groups.iter().enumerate() {
            // Skip empty groups
            if group_indices.is_empty() {
                continue;
            }

            let users: Vec<ID> = group_indices
                .iter()
                .map(|&idx| match &voter_ids[idx] {
                    VoterId::User(uuid) => uuid.into(),
                    VoterId::Session(session) => ID::from(session.to_string()),
                })
                .collect();

            let characteristic_votes: Vec<CharacteristicVote> =
                analyze_group_votes(&matrix, group_indices, &statement_ids);

            let summary = match generate_group_summary(&db_pool, &characteristic_votes).await {
                Ok(summary) => summary,
                Err(e) => {
                    eprintln!("Error generating group summary: {:?}", e);
                    "Group summary unavailable.".to_string()
                }
            };

            opinion_groups.push(OpinionGroup {
                id: ID::from(group_id.to_string()),
                users,
                characteristic_votes,
                summary: summary.to_string(),
            });
        }

        Ok(opinion_groups)
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

    async fn percent_voted(&self, ctx: &Context<'_>) -> Result<f64> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let total_votes = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM statement_vote
            WHERE statement_id = $1
            "#,
            Uuid::parse_str(&self.id.to_string())?
        )
        .fetch_one(&db_pool)
        .await?
        .count;

        let total_participants = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT COALESCE(user_id, session_id)) as "count!"
            FROM statement_vote
            WHERE statement_id = $1
            "#,
            Uuid::parse_str(&self.id.to_string())?
        )
        .fetch_one(&db_pool)
        .await?
        .count;

        Ok((total_votes as f64 / total_participants as f64) * 100.0)
    }
}

fn analyze_group_votes(
    matrix: &Array2<f64>,
    group_user_indices: &[usize],
    statement_ids: &[Uuid],
) -> Vec<CharacteristicVote> {
    let mut candidates = Vec::new();

    // First pass: collect all potential characteristic votes with their metrics
    for (stmt_idx, &stmt_id) in statement_ids.iter().enumerate() {
        let votes = matrix.column(stmt_idx);

        let group_votes: Vec<f64> = group_user_indices
            .iter()
            .map(|&user_idx| votes[user_idx])
            .filter(|&vote| vote != 0.0)
            .collect();

        if !group_votes.is_empty() {
            let mean = group_votes.iter().sum::<f64>() / group_votes.len() as f64;

            let variance = if group_votes.len() > 1 {
                group_votes.iter().map(|&x| (x - mean).powi(2)).sum::<f64>()
                    / (group_votes.len() - 1) as f64
            } else {
                0.0
            };
            let std_dev = variance.sqrt();
            let mut consensus = 1.0 - (std_dev / 1.0).min(1.0);
            let significance = group_votes.len() as f64 / group_user_indices.len() as f64;

            // Adjust consensus score for high-consensus, low-participation cases
            if consensus > 0.7 && significance <= 0.5 {
                consensus = consensus * (significance / 0.5);
            }

            candidates.push((stmt_id, mean, consensus, significance));
        }
    }

    // Sort candidates by combined score (consensus * significance)
    candidates.sort_by(|a, b| {
        let score_a = a.2 * a.3; // consensus * significance
        let score_b = b.2 * b.3;
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top 10 votes or adjust thresholds to get at least some characteristic votes
    let mut characteristic_votes = Vec::new();
    let initial_consensus_threshold = 0.3;
    let initial_significance_threshold = 0.5;
    let mut consensus_threshold = initial_consensus_threshold;
    let mut significance_threshold = initial_significance_threshold;

    while characteristic_votes.is_empty()
        && consensus_threshold > 0.0
        && significance_threshold > 0.0
    {
        characteristic_votes = candidates
            .iter()
            .filter(|&(_, _, consensus, significance)| {
                *consensus >= consensus_threshold && *significance >= significance_threshold
            })
            .take(10)
            .map(
                |&(stmt_id, mean, consensus, significance)| CharacteristicVote {
                    statement_id: ID::from(stmt_id),
                    mean_sentiment: mean,
                    consensus_level: consensus,
                    significance_level: significance,
                },
            )
            .collect();

        // Reduce thresholds if no votes meet criteria
        if characteristic_votes.is_empty() {
            consensus_threshold *= 0.8;
            significance_threshold *= 0.8;
        }
    }

    // If still no votes meet thresholds, take top 3 votes regardless of thresholds
    if characteristic_votes.is_empty() {
        characteristic_votes = candidates
            .iter()
            .take(3)
            .map(
                |&(stmt_id, mean, consensus, significance)| CharacteristicVote {
                    statement_id: ID::from(stmt_id),
                    mean_sentiment: mean,
                    consensus_level: consensus,
                    significance_level: significance,
                },
            )
            .collect();
    }

    characteristic_votes
}

#[derive(Hash, Eq, PartialEq, Clone, Ord, PartialOrd, Debug)]
enum VoterId {
    User(Uuid),
    Session(Uuid),
}

// Helper function to prepare the voting matrix
fn prepare_voting_matrix(votes: &[StatementVote]) -> (Array2<f64>, Vec<VoterId>, Vec<Uuid>) {
    let mut unique_voters = std::collections::HashSet::new();
    let mut unique_statements = std::collections::HashSet::new();

    // Collect unique voters and statements
    for vote in votes {
        if let Some(user_id) = &vote.user_id {
            unique_voters.insert(VoterId::User(*user_id));
        } else if let Some(session_id) = &vote.session_id {
            unique_voters.insert(VoterId::Session(session_id.clone()));
        }
        unique_statements.insert(vote.statement_id);
    }

    let voter_ids: Vec<VoterId> = unique_voters.into_iter().collect();
    let statement_ids: Vec<Uuid> = unique_statements.into_iter().collect();

    // Create vote matrix
    let mut matrix = Array2::zeros((voter_ids.len(), statement_ids.len()));

    for vote in votes {
        let voter_idx = voter_ids
            .iter()
            .position(|v| match (v, &vote.user_id, &vote.session_id) {
                (VoterId::User(id), Some(user_id), _) => id == user_id,
                (VoterId::Session(id), _, Some(session_id)) => id == session_id,
                _ => false,
            })
            .unwrap();
        let statement_idx = statement_ids
            .iter()
            .position(|&id| id == vote.statement_id)
            .unwrap();
        let value = vote.vote_type.as_f64();
        matrix[[voter_idx, statement_idx]] = value;
    }

    (matrix, voter_ids, statement_ids)
}

fn calculate_optimal_clusters(data: &Array2<f64>) -> usize {
    const MIN_CLUSTERS: usize = 2;
    const MAX_CLUSTERS: usize = 5;
    let n_samples = data.shape()[0];

    // Early return for small datasets
    if n_samples < MAX_CLUSTERS * 2 {
        return MIN_CLUSTERS;
    }

    let mut best_k = MIN_CLUSTERS;
    let mut best_score = f64::NEG_INFINITY;

    // Use silhouette score to find optimal k
    for k in MIN_CLUSTERS..=MAX_CLUSTERS {
        let groups = kmeans(data, k, 100);

        // Calculate average cluster size and variance
        let avg_size = n_samples as f64 / k as f64;
        let size_variance = groups
            .iter()
            .map(|g| (g.len() as f64 - avg_size).powi(2))
            .sum::<f64>()
            / k as f64;

        // Calculate inter-cluster distance
        let score = calculate_cluster_quality(data, &groups, size_variance);

        if score > best_score {
            best_score = score;
            best_k = k;
        }
    }

    best_k
}

fn kmeans(data: &Array2<f64>, k: usize, max_iters: usize) -> Vec<Vec<usize>> {
    let n_samples = data.shape()[0];
    let n_features = data.shape()[1];
    let min_cluster_size = n_samples / (k * 2);
    let mut rng = rand::thread_rng();

    // Initialize centroids using k-means++ method
    let mut centroids = Array2::zeros((k, n_features));
    let mut chosen_indices = Vec::with_capacity(k);

    // Choose first centroid randomly
    let first_idx = rng.gen_range(0..n_samples);
    centroids.row_mut(0).assign(&data.row(first_idx));
    chosen_indices.push(first_idx);

    // Choose remaining centroids
    for i in 1..k {
        let mut distances = vec![f64::INFINITY; n_samples];

        // Calculate distances to existing centroids
        for sample_idx in 0..n_samples {
            for &centroid_idx in &chosen_indices {
                let dist = squared_distance(&data.row(sample_idx), &data.row(centroid_idx));
                distances[sample_idx] = distances[sample_idx].min(dist);
            }
        }

        // Choose next centroid with probability proportional to distance
        let total_dist: f64 = distances.iter().sum();
        let mut cumsum = 0.0;
        let threshold = rng.gen::<f64>() * total_dist;

        let next_idx = distances
            .iter()
            .enumerate()
            .find(|(_, &dist)| {
                cumsum += dist;
                cumsum >= threshold
            })
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| rng.gen_range(0..n_samples));

        centroids.row_mut(i).assign(&data.row(next_idx));
        chosen_indices.push(next_idx);
    }

    let mut groups = vec![Vec::new(); k];
    let mut converged = false;
    let mut iterations = 0;

    while !converged && iterations < max_iters {
        // Clear previous groups
        groups.iter_mut().for_each(|g| g.clear());

        // Assign points to nearest centroid
        for i in 0..n_samples {
            let mut min_dist = f64::INFINITY;
            let mut closest_cluster = 0;

            for j in 0..k {
                let dist = squared_distance(&data.row(i), &centroids.row(j));
                if dist < min_dist {
                    min_dist = dist;
                    closest_cluster = j;
                }
            }

            groups[closest_cluster].push(i);
        }

        // Rebalance clusters if needed
        rebalance_clusters(&mut groups, min_cluster_size);

        // Update centroids and check convergence
        let mut max_centroid_shift: f64 = 0.0;
        for (i, group) in groups.iter().enumerate() {
            if !group.is_empty() {
                let new_centroid = calculate_centroid(data, group);
                let shift = squared_distance(&centroids.row(i), &new_centroid.row(0));
                max_centroid_shift = max_centroid_shift.max(shift);
                centroids.row_mut(i).assign(&new_centroid.row(0));
            }
        }

        converged = max_centroid_shift < 1e-4;
        iterations += 1;
    }

    groups
}

fn squared_distance(a: &ArrayView1<f64>, b: &ArrayView1<f64>) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

fn calculate_centroid(data: &Array2<f64>, indices: &[usize]) -> Array2<f64> {
    let sum = indices
        .iter()
        .fold(Array2::zeros((1, data.shape()[1])), |acc, &idx| {
            acc + data.row(idx).insert_axis(Axis(0))
        });
    sum / indices.len() as f64
}

fn rebalance_clusters(groups: &mut Vec<Vec<usize>>, min_size: usize) {
    let mut large_clusters: Vec<usize> = groups
        .iter()
        .enumerate()
        .filter(|(_, g)| g.len() > min_size * 2)
        .map(|(i, _)| i)
        .collect();

    let mut small_clusters: Vec<usize> = groups
        .iter()
        .enumerate()
        .filter(|(_, g)| g.len() < min_size)
        .map(|(i, _)| i)
        .collect();

    while !large_clusters.is_empty() && !small_clusters.is_empty() {
        let from = large_clusters[0];
        let to = small_clusters[0];

        let n_transfer = (groups[from].len() - min_size).min(min_size - groups[to].len());
        let transfer_elements: Vec<_> = groups[from].drain(0..n_transfer).collect();
        groups[to].extend(transfer_elements);

        if groups[from].len() <= min_size * 2 {
            large_clusters.remove(0);
        }
        if groups[to].len() >= min_size {
            small_clusters.remove(0);
        }
    }
}

fn calculate_cluster_quality(data: &Array2<f64>, groups: &[Vec<usize>], size_variance: f64) -> f64 {
    let mut total_score = 0.0;
    let size_penalty = 1.0 / (1.0 + size_variance.sqrt());

    for (i, group1) in groups.iter().enumerate() {
        if group1.is_empty() {
            continue;
        }

        let mut min_inter_dist = f64::INFINITY;

        for (j, group2) in groups.iter().enumerate() {
            if i == j || group2.is_empty() {
                continue;
            }

            let dist = calculate_group_distance(data, group1, group2);
            min_inter_dist = min_inter_dist.min(dist);
        }

        let intra_dist = calculate_group_cohesion(data, group1);
        if intra_dist == 0.0 {
            continue;
        }

        total_score += (min_inter_dist / intra_dist) * size_penalty;
    }

    total_score / groups.len() as f64
}

fn calculate_group_distance(data: &Array2<f64>, group1: &[usize], group2: &[usize]) -> f64 {
    let mut total_dist = 0.0;
    let mut count = 0;

    for &i in group1 {
        for &j in group2 {
            total_dist += squared_distance(&data.row(i), &data.row(j));
            count += 1;
        }
    }

    if count > 0 {
        total_dist / count as f64
    } else {
        f64::INFINITY
    }
}

fn calculate_group_cohesion(data: &Array2<f64>, group: &[usize]) -> f64 {
    if group.len() <= 1 {
        return 1.0;
    }

    let mut total_dist = 0.0;
    let mut count = 0;

    for (i, &idx1) in group.iter().enumerate() {
        for &idx2 in group.iter().skip(i + 1) {
            total_dist += squared_distance(&data.row(idx1), &data.row(idx2));
            count += 1;
        }
    }

    if count > 0 {
        total_dist / count as f64
    } else {
        f64::INFINITY
    }
}

#[derive(Clone)]
struct StatementWithMeta {
    id: Uuid,
    content: String,
    votes: Vec<StatementVote>,
    views: Vec<StatementView>,
}

async fn fetch_statements_with_votes(
    db_pool: &PgPool,
    conversation_id: &str,
) -> Result<Vec<StatementWithMeta>> {
    let votes = sqlx::query_as!(
        StatementVote,
        r#"
        SELECT v.id, statement_id, s.content, user_id, session_id, vote_type AS "vote_type: ArgumentPosition", v.created_at, v.updated_at
        FROM statement_vote v
        JOIN statement s ON v.statement_id = s.id
        WHERE s.conversation_id = $1
        "#,
        Uuid::parse_str(conversation_id)?
    )
    .fetch_all(db_pool)
    .await?;

    let views = sqlx::query_as!(
        StatementView,
        r#"
        SELECT sv.id, statement_id, session_id, user_id, sv.created_at, sv.updated_at
        FROM statement_view sv
        JOIN statement s ON sv.statement_id = s.id
        WHERE s.conversation_id = $1
        "#,
        Uuid::parse_str(conversation_id)?
    )
    .fetch_all(db_pool)
    .await?;

    let mut statements_map: HashMap<Uuid, StatementWithMeta> = HashMap::new();

    // Process votes first to establish content
    for vote in votes {
        statements_map
            .entry(vote.statement_id)
            .or_insert_with(|| StatementWithMeta {
                id: vote.statement_id,
                content: vote.content.clone(),
                votes: Vec::new(),
                views: Vec::new(),
            })
            .votes
            .push(vote);
    }

    // Process views, skipping statements that don't exist
    for view in views {
        if let Some(statement) = statements_map.get_mut(&view.statement_id) {
            statement.views.push(view);
        }
    }

    Ok(statements_map.into_values().collect())
}

fn count_votes(votes: &[StatementVote]) -> HashMap<ArgumentPosition, i32> {
    let mut counts = HashMap::new();
    for vote in votes {
        *counts.entry(vote.vote_type.clone()).or_insert(0) += 1;
    }
    counts
}

fn calculate_consensus_score(
    vote_counts: &HashMap<ArgumentPosition, i32>,
    total_votes: f64,
) -> f64 {
    let support = *vote_counts.get(&ArgumentPosition::Support).unwrap_or(&0) as f64;
    let oppose = *vote_counts.get(&ArgumentPosition::Oppose).unwrap_or(&0) as f64;
    let neutral = *vote_counts.get(&ArgumentPosition::Neutral).unwrap_or(&0) as f64;

    let max_votes = support.max(oppose).max(neutral);
    let max_vote_ratio = max_votes / total_votes;

    // Penalize statements with few votes
    let vote_volume_factor = (total_votes / 10.0).min(1.0);

    max_vote_ratio * vote_volume_factor
}

fn calculate_divisiveness_score(
    vote_counts: &HashMap<ArgumentPosition, i32>,
    total_votes: f64,
) -> f64 {
    let support = vote_counts
        .get(&ArgumentPosition::Support)
        .copied()
        .unwrap_or(0) as f64;
    let oppose = vote_counts
        .get(&ArgumentPosition::Oppose)
        .copied()
        .unwrap_or(0) as f64;

    // Ignore neutral votes for divisiveness calculation
    let active_votes = support + oppose;
    if active_votes == 0.0 {
        return 0.0;
    }

    // Calculate the proportion of support vs oppose among non-neutral votes
    let support_ratio = support / active_votes;

    // Score is highest when support_ratio is close to 0.5 (perfect split)
    // and lowest when it's close to 0.0 or 1.0 (consensus)
    let balance_score = 1.0 - (support_ratio - 0.5).abs() * 2.0;

    // Consider total engagement (non-neutral votes) as a factor
    let engagement_ratio = active_votes / total_votes;

    // Penalize low vote counts
    let vote_volume_factor = (total_votes / 10.0).min(1.0);

    // Combine factors with more weight on the balance score
    balance_score * engagement_ratio * vote_volume_factor
}

async fn generate_group_summary(
    db_pool: &PgPool,
    characteristic_votes: &[CharacteristicVote],
) -> Result<String, Error> {
    // Fetch statement contents for context
    let mut statement_details = Vec::new();
    for vote in characteristic_votes {
        let statement = sqlx::query!(
            r#"
            SELECT content
            FROM statement
            WHERE id = $1
            "#,
            Uuid::parse_str(&vote.statement_id)?
        )
        .fetch_one(db_pool)
        .await?;

        statement_details.push((
            statement.content,
            vote.mean_sentiment,
            vote.consensus_level,
            vote.significance_level,
        ));
    }

    let prompt = format!(
        "You are analyzing a group of users in a discussion. Here are their most characteristic voting patterns:\n\n{}{}",
        statement_details.iter()
            .map(|(content, sentiment, consensus, significance)| {
                format!(
                    "Statement: '{}'\nSentiment: {:.2} (-1 to +1)\nConsensus: {:.2}\nParticipation: {:.2}\n",
                    content, sentiment, consensus, significance
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "\nBased on these voting patterns, write a 2-3 sentence summary describing this group's positions and characteristics. Focus on the statement content and the most strongly held views and areas of agreement."
    );

    dotenv::dotenv().ok();
    let client = Client::new();

    let response = client
        .chat()
        .create(
            CreateChatCompletionRequestArgs::default()
                .model("gpt-4")
                .messages([ChatCompletionRequestMessage {
                    role: Role::User,
                    content: prompt,
                    name: None,
                }])
                .temperature(0.7)
                .max_tokens(40_u16)
                .build()?,
        )
        .await;

    // Handle API error
    match response {
        Ok(response) => {
            // Extract and return the generated summary
            let summary = response.choices[0].message.content.clone();

            Ok(summary)
        }
        Err(err) => Err(Error::new(format!("OpenAI API error: {:?}", err)))?,
    }
}

async fn generate_opinion_summary(
    consensus_opinions: Vec<OpinionScore>,
    divisive_opinions: Vec<OpinionScore>,
) -> Result<String, Error> {
    if consensus_opinions.is_empty() && divisive_opinions.is_empty() {
        return Ok("No opinions to summarize.".to_string());
    }

    let mut opinion_details = Vec::new();

    // Format consensus opinions
    for opinion in consensus_opinions {
        opinion_details.push(format!(
            "Consensus Opinion: '{}'\nSupport: {}\nOppose: {}\nNeutral: {}\nTotal Votes: {}\n",
            opinion.content,
            opinion.support_votes,
            opinion.oppose_votes,
            opinion.neutral_votes,
            opinion.total_votes
        ));
    }

    // Format divisive opinions
    for opinion in divisive_opinions {
        opinion_details.push(format!(
            "Divisive Opinion: '{}'\nSupport: {}\nOppose: {}\nNeutral: {}\nTotal Votes: {}\n",
            opinion.content,
            opinion.support_votes,
            opinion.oppose_votes,
            opinion.neutral_votes,
            opinion.total_votes
        ));
    }

    let prompt = format!(
        "You are analyzing voting patterns on various opinions. Here are the most notable consensus and divisive opinions:\n\n{}{}",
        opinion_details.join("\n"),
        "\nWrite a 2-3 sentence summary describing the overall patterns in these opinions. Focus on what unites and divides the community, without directly quoting the statements. Highlight any particularly strong consensus or notable divisions."
    );

    dotenv::dotenv().ok();
    let client = Client::new();

    let response = client
        .chat()
        .create(
            CreateChatCompletionRequestArgs::default()
                .model("gpt-4")
                .messages([ChatCompletionRequestMessage {
                    role: Role::User,
                    content: prompt,
                    name: None,
                }])
                .temperature(0.7)
                .max_tokens(320_u16)
                .build()?,
        )
        .await;

    match response {
        Ok(response) => Ok(response.choices[0].message.content.clone()),
        Err(err) => Err(Error::new(format!("OpenAI API error: {:?}", err))),
    }
}
