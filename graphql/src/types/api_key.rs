use async_graphql::{SimpleObject, ID};
use db::ApiKey;

#[derive(Debug, Clone, SimpleObject)]
pub struct ApiKeyResult {
    pub id: ID,
    pub name: String,
    pub prefix: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<ApiKey> for ApiKeyResult {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id.into(),
            name: key.name,
            prefix: key.key_prefix,
            created_at: key.created_at,
            last_used_at: key.last_used_at,
        }
    }
}

#[derive(Debug, SimpleObject)]
pub struct CreatedApiKeyResult {
    pub api_key: ApiKeyResult,
    /// The secret is returned only once and cannot be recovered later.
    pub key: String,
}
