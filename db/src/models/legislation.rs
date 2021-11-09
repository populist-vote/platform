use async_graphql::Enum;

use crate::DateTime;

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "vote_status", rename_all = "lowercase")]
pub enum LegislationStatus {
    INTRODUCED,
    PASSED,
    SIGNED,
    VETOED,
    UNKNOWN,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Legislation {
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
    pub vote_status: LegislationStatus,
    pub description: Option<String>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    // public votes
    // issue tags
    // arguments -> use graphQL ...on Type
    // events
    // sponsors
    // related organizations
    // news
    // mentions
    // related legislation
}
