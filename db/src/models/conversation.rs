use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

use crate::ArgumentPosition;

#[derive(FromRow, Clone)]
pub struct Conversation {
    pub id: uuid::Uuid,
    pub prompt: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, SimpleObject, Clone)]
pub struct Statement {
    pub id: uuid::Uuid,
    pub conversation_id: uuid::Uuid,
    pub content: String,
    pub author_id: Option<uuid::Uuid>, // Optional author ID for anonymous statements
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, SimpleObject, Clone)]
pub struct StatementVote {
    id: uuid::Uuid,
    statement_id: uuid::Uuid,
    participant_id: Option<uuid::Uuid>, // Optional participant ID for anonymous votes
    vote_type: ArgumentPosition,
    created_at: DateTime<Utc>,
}
