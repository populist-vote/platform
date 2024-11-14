use async_graphql::{ComplexObject, Context, SimpleObject, ID};
use chrono::{DateTime, Utc};
use db::{models::conversation::Conversation, UserWithProfile};

use crate::context::ApiContext;

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
    prompt: String,
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

impl From<Conversation> for ConversationResult {
    fn from(conversation: Conversation) -> Self {
        Self {
            id: ID(conversation.id.to_string()),
            prompt: conversation.prompt,
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
}
