use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::{ArgumentPosition, StatementModerationStatus};

#[derive(FromRow, Clone)]
pub struct Conversation {
    pub id: uuid::Uuid,
    pub topic: String,
    pub description: Option<String>,
    pub organization_id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, SimpleObject, Clone)]
pub struct Statement {
    pub id: uuid::Uuid,
    pub conversation_id: uuid::Uuid,
    pub content: String,
    pub author_id: Option<uuid::Uuid>, // Optional author ID for anonymous statements
    pub moderation_status: StatementModerationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, SimpleObject, Clone, Debug)]
pub struct StatementVote {
    pub id: uuid::Uuid,
    pub content: String,
    pub statement_id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub session_id: Option<uuid::Uuid>,
    pub vote_type: ArgumentPosition,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, SimpleObject, sqlx::FromRow, Clone)]
pub struct StatementView {
    pub id: uuid::Uuid,
    pub statement_id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
