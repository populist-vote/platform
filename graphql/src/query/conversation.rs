use async_graphql::{Context, Object, ID};
use db::{models::conversation::Conversation, OrganizationRoleType};

use crate::{context::ApiContext, guard::OrganizationGuard, is_admin, types::ConversationResult};

#[derive(Default)]
pub struct ConversationQuery;

#[Object]
impl ConversationQuery {
    #[graphql(
        guard = "OrganizationGuard::new(&organization_id, &OrganizationRoleType::ReadOnly)",
        visible = "is_admin"
    )]
    async fn conversations_by_organization(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
        limit: Option<i64>,
    ) -> async_graphql::Result<Vec<ConversationResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let limit = limit.unwrap_or(10);
        let conversations = sqlx::query_as!(
            Conversation,
            "SELECT * FROM conversation WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2",
            uuid::Uuid::parse_str(&organization_id)?,
            limit
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(conversations.into_iter().map(|c| c.into()).collect())
    }

    #[graphql(visible = "is_admin")]
    async fn conversation_by_id(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<ConversationResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let conversation = sqlx::query_as!(
            Conversation,
            "SELECT * FROM conversation WHERE id = $1",
            uuid::Uuid::parse_str(&id)?,
        )
        .fetch_optional(&db_pool)
        .await?;

        Ok(conversation.map(|c| c.into()))
    }
}
