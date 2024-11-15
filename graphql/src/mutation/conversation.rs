use async_graphql::{Context, Error, InputObject, Object, ID};
use auth::AccessTokenClaims;
use db::{
    models::conversation::{Conversation, Statement, StatementVote},
    ArgumentPosition,
};
use jsonwebtoken::TokenData;
use uuid::Uuid;

use crate::{context::ApiContext, types::ConversationResult, SessionData};

#[derive(Default)]
pub struct ConversationMutation;

#[derive(InputObject)]
struct CreateConversationInput {
    prompt: String,
    description: Option<String>,
}

#[derive(InputObject)]
struct AddStatementInput {
    conversation_id: ID,
    content: String,
    user_id: Option<ID>,
}

#[Object]
impl ConversationMutation {
    async fn create_conversation(
        &self,
        ctx: &Context<'_>,
        input: CreateConversationInput,
    ) -> async_graphql::Result<ConversationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let conversation = sqlx::query_as!(
            Conversation,
            r#"
            INSERT INTO conversation (
                prompt, 
                description, 
                created_at, 
                updated_at
            )
            VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING *
            "#,
            input.prompt,
            input.description
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(conversation.into())
    }

    async fn add_statement(
        &self,
        ctx: &Context<'_>,
        input: AddStatementInput,
    ) -> async_graphql::Result<Statement> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        // Verify conversation exists
        let conversation_id = Uuid::parse_str(&input.conversation_id)
            .map_err(|_| Error::new("Invalid conversation ID"))?;

        let author_id = match input.user_id {
            Some(user_id) => Some(
                Uuid::parse_str(&user_id.to_string()).map_err(|_| Error::new("Invalid user ID"))?,
            ),
            None => match ctx.data::<Option<TokenData<AccessTokenClaims>>>() {
                Ok(token_data) => {
                    let user_id = token_data.as_ref().map(|token_data| token_data.claims.sub);
                    match user_id {
                        Some(user_id) => Some(user_id),
                        None => None,
                    }
                }
                Err(_) => None,
            },
        };

        let statement = sqlx::query_as!(
            Statement,
            r#"
            INSERT INTO statement (
                conversation_id,
                content,
                author_id
            )
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            conversation_id,
            input.content,
            author_id
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(statement)
    }

    async fn vote_on_statement(
        &self,
        ctx: &Context<'_>,
        statement_id: ID,
        user_id: Option<ID>,
        vote_type: ArgumentPosition,
    ) -> async_graphql::Result<StatementVote> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let session_data = ctx.data::<SessionData>()?.clone();
        let session_id = session_data.session_id;

        let statement_id = Uuid::parse_str(&statement_id.to_string())
            .map_err(|_| Error::new("Invalid statement ID"))?;

        let user_id = match user_id {
            Some(user_id) => Some(
                Uuid::parse_str(&user_id.to_string()).map_err(|_| Error::new("Invalid user ID"))?,
            ),
            None => match ctx.data::<TokenData<AccessTokenClaims>>() {
                Ok(token_data) => Some(token_data.claims.sub),
                Err(_) => None,
            },
        };

        // Verify statement exists
        sqlx::query!("SELECT id FROM statement WHERE id = $1", statement_id)
            .fetch_one(&db_pool)
            .await
            .map_err(|_| Error::new("Statement not found"))?;

        // Upsert vote
        let vote = sqlx::query_as::<_, StatementVote>(
            r#"
            INSERT INTO statement_vote (
                statement_id,
                user_id,
                session_id,
                vote_type,
                created_at
            )
            VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
            ON CONFLICT (statement_id, participant_id)
            DO UPDATE SET 
                vote_type = EXCLUDED.vote_type,
                created_at = CURRENT_TIMESTAMP
            RETURNING *
            "#,
        )
        .bind(statement_id)
        .bind(user_id)
        .bind(uuid::Uuid::parse_str(&session_id.to_string())?)
        .bind(vote_type)
        .fetch_one(&db_pool)
        .await?;

        Ok(vote)
    }
}
