use crate::DateTime;
use serde_json::Value as JSON;

#[derive(sqlx::FromRow, Debug, Clone, Eq, PartialEq)]
pub struct OrganizationPoliticianNote {
    pub id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub politician_id: uuid::Uuid,
    pub election_id: uuid::Uuid,
    pub issue_tag_ids: Vec<uuid::Uuid>,
    pub notes: JSON,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
