use crate::DateTime;
use sqlx::FromRow;

use super::enums::State;

#[derive(FromRow, Debug, Clone)]
pub struct Committee {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub state: Option<State>,
    pub chair_id: Option<uuid::Uuid>,
    pub legiscan_committee_id: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
