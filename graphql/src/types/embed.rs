use async_graphql::{SimpleObject, ID};
use chrono::{DateTime, Utc};
use db::Embed;
use serde_json::Value as JSON;

#[derive(SimpleObject, Clone, Debug)]
pub struct EmbedResult {
    pub id: ID,
    pub organization_id: ID,
    pub name: String,
    pub description: Option<String>,
    pub populist_url: String,
    pub attributes: JSON,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Embed> for EmbedResult {
    fn from(embed: Embed) -> Self {
        Self {
            id: embed.id.into(),
            organization_id: embed.organization_id.into(),
            name: embed.name,
            description: embed.description,
            populist_url: embed.populist_url,
            attributes: embed.attributes,
            created_at: embed.created_at,
            updated_at: embed.updated_at,
        }
    }
}
