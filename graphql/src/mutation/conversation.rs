use async_graphql::{Context, Error, InputObject, Object, Result, ID};
use auth::AccessTokenClaims;

use db::{
    models::conversation::{Conversation, Statement, StatementView, StatementVote},
    ArgumentPosition,
};
use jsonwebtoken::TokenData;
use uuid::Uuid;

use crate::{context::ApiContext, types::ConversationResult, SessionData};

#[derive(Default)]
pub struct ConversationMutation;

#[derive(Default)]
pub struct StatementMutation;

#[derive(InputObject)]
struct CreateConversationInput {
    topic: String,
    description: Option<String>,
    organization_id: ID,
    seed_statements: Option<Vec<String>>,
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
            WITH new_conversation AS (
                INSERT INTO conversation (
                    topic, 
                    description, 
                    organization_id,
                    created_at, 
                    updated_at
                )
                VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                RETURNING *
            ),
            statement_insert AS (
                INSERT INTO statement (
                    conversation_id,
                    content,
                    created_at,
                    updated_at
                )
                SELECT 
                    (SELECT id FROM new_conversation),
                    unnest($4::text[]),
                    CURRENT_TIMESTAMP,
                    CURRENT_TIMESTAMP
                WHERE array_length($4::text[], 1) > 0
            )
            SELECT * FROM new_conversation
            "#,
            input.topic,
            input.description,
            Uuid::parse_str(&input.organization_id.to_string())
                .map_err(|_| Error::new("Invalid organization ID"))?,
            input
                .seed_statements
                .as_ref()
                .map(|seed_statements| seed_statements.as_slice()),
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(conversation.into())
    }

    async fn update_conversation(
        &self,
        ctx: &Context<'_>,
        conversation_id: ID,
        topic: Option<String>,
        description: Option<String>,
    ) -> async_graphql::Result<ConversationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let conversation_id = Uuid::parse_str(&conversation_id.to_string())
            .map_err(|_| Error::new("Invalid conversation ID"))?;

        let conversation = sqlx::query_as!(
            Conversation,
            r#"
            UPDATE conversation
            SET
                topic = COALESCE($2, topic),
                description = COALESCE($3, description),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING *
            "#,
            conversation_id,
            topic,
            description
        )
        .fetch_one(&db_pool)
        .await?
        .into();

        Ok(conversation)
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
        let session_id = uuid::Uuid::parse_str(&session_data.session_id.to_string())?;

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

        // Use different queries based on whether we have a user_id
        let vote = if let Some(user_id) = user_id {
            // Use user_id for conflict
            sqlx::query_as!(
                StatementVote,
                r#"
                INSERT INTO statement_vote (
                    statement_id,
                    user_id,
                    session_id,
                    vote_type
                )
                VALUES ($1, $2, $3, $4::argument_position)
                ON CONFLICT (statement_id, user_id)
                DO UPDATE SET 
                    vote_type = EXCLUDED.vote_type,
                    session_id = EXCLUDED.session_id,
                    updated_at = CURRENT_TIMESTAMP
                RETURNING 
                    id,
                    statement_id,
                    user_id,
                    session_id,
                    vote_type AS "vote_type: ArgumentPosition",
                    created_at,
                    updated_at
                "#,
                statement_id,
                user_id,
                Some(session_id),
                vote_type as ArgumentPosition
            )
            .fetch_one(&db_pool)
            .await?
        } else {
            // Use session_id for conflict
            sqlx::query_as!(
                StatementVote,
                r#"
                INSERT INTO statement_vote (
                    statement_id,
                    user_id,
                    session_id,
                    vote_type
                )
                VALUES ($1, $2, $3, $4::argument_position)
                ON CONFLICT (statement_id, session_id)
                DO UPDATE SET 
                    vote_type = EXCLUDED.vote_type,
                    updated_at = CURRENT_TIMESTAMP
                RETURNING 
                    id,
                    statement_id,
                    user_id,
                    session_id,
                    vote_type AS "vote_type: ArgumentPosition",
                    created_at,
                    updated_at
                "#,
                statement_id,
                None::<Uuid>,
                session_id,
                vote_type as ArgumentPosition
            )
            .fetch_one(&db_pool)
            .await?
        };

        Ok(vote)
    }
}

#[Object]

impl StatementMutation {
    pub async fn record_view(
        &self,
        ctx: &Context<'_>,
        statement_id: ID,
        user_id: Option<ID>,
    ) -> Result<StatementView> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let session_data = ctx.data::<SessionData>()?.clone();
        let session_id = session_data.session_id;
        // Using ON CONFLICT DO NOTHING since we only want one view per session
        let view = sqlx::query_as!(
            StatementView,
            r#"
            INSERT INTO statement_view (statement_id, session_id, user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (statement_id, session_id) DO NOTHING
            RETURNING id, statement_id, session_id, user_id, created_at, updated_at
            "#,
            uuid::Uuid::parse_str(&statement_id)?,
            uuid::Uuid::parse_str(&session_id.to_string())?,
            user_id.map(|id| uuid::Uuid::parse_str(&id)).transpose()?
        )
        .fetch_optional(&db_pool)
        .await?;

        // If there was a conflict, fetch the existing view
        Ok(match view {
            Some(v) => v,
            None => {
                sqlx::query_as!(
                    StatementView,
                    r#"
                SELECT id, statement_id, session_id, user_id, created_at, updated_at
                FROM statement_view
                WHERE statement_id = $1 AND session_id = $2
                "#,
                    uuid::Uuid::parse_str(&statement_id)?,
                    uuid::Uuid::parse_str(&session_id.to_string())?
                )
                .fetch_one(&db_pool)
                .await?
            }
        })
    }
}
