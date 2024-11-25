use std::collections::{HashMap, HashSet};

use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject, ID};
use auth::AccessTokenClaims;
use chrono::{DateTime, Utc};
use db::{
    models::conversation::{Conversation, StatementView, StatementVote},
    ArgumentPosition, UserWithProfile,
};
use itertools::Itertools;
use jsonwebtoken::TokenData;
use kmeans::*;
use ndarray::Array2;
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

#[derive(SimpleObject)]
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
    consensus_opinions: Vec<OpinionScore>,
    divisive_opinions: Vec<OpinionScore>,
}

#[derive(SimpleObject)]
struct OpinionGroup {
    id: ID,
    users: Vec<ID>, // Using String to represent UUIDs
    characteristic_votes: Vec<CharacteristicVote>,
}

#[derive(SimpleObject)]
struct CharacteristicVote {
    statement_id: ID,
    mean_sentiment: f64,
    consensus_level: f64,
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

        let consensus_opinions = consensus_statements
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

        let divisive_opinions = divisive_statements
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

        Ok(OpinionAnalysis {
            consensus_opinions,
            divisive_opinions,
        })
    }

    async fn opinion_groups(
        &self,
        ctx: &Context<'_>,
        num_groups: i32,
    ) -> Result<Vec<OpinionGroup>, Error> {
        todo!("Buggy implementation, needs to be fixed and optimized");

        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        // Fetch all votes
        let votes = sqlx::query_as!(
            StatementVote,
            r#"
            SELECT v.id, statement_id, user_id, session_id, vote_type AS "vote_type: ArgumentPosition", v.created_at, v.updated_at
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

        // Perform k-means clustering
        let groups = cluster_opinions(&matrix, num_groups as usize);

        // Analyze groups
        let mut opinion_groups = Vec::new();
        for (group_id, group_indices) in groups.iter().enumerate() {
            // Convert indices back to user IDs for this group
            let users: Vec<ID> = group_indices
                .iter()
                .filter_map(|&idx| {
                    match &voter_ids[idx as usize] {
                        VoterId::User(uuid) => Some(uuid.into()),
                        VoterId::Session(_) => None, // Skip session IDs in the output
                    }
                })
                .collect();

            let characteristic_votes = analyze_group_votes(&matrix, group_indices, &statement_ids);

            opinion_groups.push(OpinionGroup {
                id: ID::from(group_id.to_string()),
                users,
                characteristic_votes,
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
}

fn cluster_opinions(matrix: &Array2<f64>, k: usize) -> Vec<Vec<i32>> {
    // Convert ndarray matrix to flat vector
    let samples: Vec<f64> = matrix.iter().cloned().collect();
    let sample_cnt = matrix.nrows();
    let sample_dims = matrix.ncols();

    // Guard against k being larger than sample count
    let k = k.min(sample_cnt);

    // Create KMeans instance
    let kmean: KMeans<f64, 8, EuclideanDistance> =
        KMeans::new(samples, sample_cnt, sample_dims, EuclideanDistance);

    // Run clustering
    let result = kmean.kmeans_lloyd(
        k,
        100, // max iterations
        KMeans::init_kmeanplusplus,
        &KMeansConfig::default(),
    );

    // Convert assignments back to our group format
    let mut groups: Vec<Vec<i32>> = vec![Vec::new(); k];
    for (idx, &cluster) in result.assignments.iter().enumerate() {
        groups[cluster].push(idx as i32);
    }

    groups
}

fn analyze_group_votes(
    matrix: &Array2<f64>,
    group_user_indices: &[i32],
    statement_ids: &[Uuid],
) -> Vec<CharacteristicVote> {
    let mut characteristic_votes = Vec::new();

    for (stmt_idx, &stmt_id) in statement_ids.iter().enumerate() {
        // Get all votes for this statement
        let votes = matrix.column(stmt_idx);

        // Calculate statistics for this group
        let group_votes: Vec<f64> = group_user_indices
            .iter()
            .map(|&user_idx| votes[user_idx as usize])
            .collect();

        // Calculate mean
        let mean = group_votes.iter().sum::<f64>() / group_votes.len() as f64;

        // Calculate standard deviation
        let variance =
            group_votes.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / group_votes.len() as f64;
        let std_dev = variance.sqrt();

        // Calculate consensus (1 - normalized std dev)
        let consensus = 1.0 - (std_dev / 2.0).min(1.0);

        characteristic_votes.push(CharacteristicVote {
            statement_id: ID::from(stmt_id.to_string()),
            mean_sentiment: mean,
            consensus_level: consensus,
        });
    }

    characteristic_votes
}

#[derive(Hash, Eq, PartialEq, Clone, Ord, PartialOrd)]
enum VoterId {
    User(Uuid),
    Session(Uuid),
}

fn prepare_voting_matrix(votes: &[StatementVote]) -> (Array2<f64>, Vec<VoterId>, Vec<Uuid>) {
    // Collect voter IDs, preferring user_id over session_id
    let voter_ids: Vec<VoterId> = votes
        .iter()
        .filter_map(|v| match (v.user_id, v.session_id) {
            (Some(user_id), _) => Some(VoterId::User(user_id)),
            (None, Some(session_id)) => Some(VoterId::Session(session_id)),
            (None, None) => None,
        })
        .collect::<HashSet<VoterId>>()
        .into_iter()
        .sorted()
        .collect();

    let statement_ids: Vec<Uuid> = votes
        .iter()
        .map(|v| v.statement_id)
        .collect::<HashSet<Uuid>>()
        .into_iter()
        .sorted()
        .collect();

    // Create a 2D array filled with zeros
    let mut matrix = Array2::zeros((voter_ids.len(), statement_ids.len()));

    // Fill the matrix with votes
    for vote in votes {
        let voter_id = match (vote.user_id, vote.session_id) {
            (Some(user_id), _) => Some(VoterId::User(user_id)),
            (None, Some(session_id)) => Some(VoterId::Session(session_id)),
            (None, None) => None,
        };

        if let Some(voter_id) = voter_id {
            if let Some(voter_idx) = voter_ids.iter().position(|id| id == &voter_id) {
                if let Some(stmt_idx) = statement_ids.iter().position(|&id| id == vote.statement_id)
                {
                    matrix[[voter_idx, stmt_idx]] = match vote.vote_type {
                        ArgumentPosition::Support => 1.0,
                        ArgumentPosition::Oppose => -1.0,
                        ArgumentPosition::Neutral => 0.0,
                    };
                }
            }
        }
    }

    (matrix, voter_ids, statement_ids)
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
    let statements = sqlx::query!(
        r#"
        SELECT id, content
        FROM statement
        WHERE conversation_id = $1
        "#,
        Uuid::parse_str(conversation_id)?
    )
    .fetch_all(db_pool)
    .await?;

    let votes = sqlx::query_as!(
        StatementVote,
        r#"
        SELECT v.id, statement_id, user_id, session_id, vote_type AS "vote_type: ArgumentPosition", v.created_at, v.updated_at
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

    let mut votes_by_statement: HashMap<Uuid, Vec<StatementVote>> = HashMap::new();
    for vote in votes {
        votes_by_statement
            .entry(vote.statement_id)
            .or_default()
            .push(vote);
    }

    let mut views_by_statement: HashMap<Uuid, Vec<StatementView>> = HashMap::new();
    for view in views {
        views_by_statement
            .entry(view.statement_id)
            .or_default()
            .push(view);
    }

    Ok(statements
        .into_iter()
        .map(|statement| StatementWithMeta {
            id: statement.id,
            content: statement.content,
            votes: votes_by_statement.remove(&statement.id).unwrap_or_default(),
            views: views_by_statement.remove(&statement.id).unwrap_or_default(),
        })
        .collect())
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

#[test]
fn test_clustering() {
    // Create a simple test matrix where we expect clear clusters
    let mut matrix = Array2::zeros((10, 3));

    // First group: all support
    matrix.slice_mut(ndarray::s![0..3, ..]).fill(1.0);

    // Second group: all oppose
    matrix.slice_mut(ndarray::s![3..6, ..]).fill(-1.0);

    // Third group: all neutral
    matrix.slice_mut(ndarray::s![6..10, ..]).fill(0.0);

    let groups = cluster_opinions(&matrix, 3);

    assert_eq!(groups.len(), 3);
    // Further assertions about group composition...
}
