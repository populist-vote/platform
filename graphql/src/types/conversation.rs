use async_graphql::{ComplexObject, Context, SimpleObject, ID};
use chrono::{DateTime, Utc};
use db::models::conversation::Conversation;

use crate::context::ApiContext;

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
struct StatementResult {
    id: ID,
    conversation_id: ID,
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
