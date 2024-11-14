use async_graphql::{Context, Object, ID};
use db::models::conversation::Conversation;

use crate::{context::ApiContext, types::ConversationResult};

#[derive(Default)]
pub struct ConversationQuery;

#[Object]
impl ConversationQuery {
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
