use async_graphql::*;
use db::{
    models::vote::{VotableType, Vote, VoteDirection},
    Argument, UpdateArgumentInput,
};
use sqlx::{Pool, Postgres};

use crate::types::ArgumentResult;
#[derive(Default)]
pub struct ArgumentMutation;

#[derive(SimpleObject)]
struct DeleteArgumentResult {
    id: String,
}

#[Object]
impl ArgumentMutation {
    async fn update_argument(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateArgumentInput,
    ) -> Result<ArgumentResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let updated_record = Argument::update(db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(ArgumentResult::from(updated_record))
    }

    async fn delete_argument(&self, ctx: &Context<'_>, id: String) -> Result<DeleteArgumentResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        Argument::delete(db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteArgumentResult { id })
    }

    async fn upvote_argument(
        &self,
        ctx: &Context<'_>,
        argument_id: ID,
        populist_user_id: ID,
    ) -> Result<bool> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let vote = Vote {
            populist_user_id: uuid::Uuid::parse_str(&populist_user_id)?,
            votable_id: uuid::Uuid::parse_str(&argument_id)?,
            votable_type: VotableType::Argument,
            direction: VoteDirection::UP,
        };
        Vote::upsert(db_pool, vote).await?;

        Ok(true)
    }

    async fn downvote_argument(
        &self,
        ctx: &Context<'_>,
        argument_id: ID,
        populist_user_id: ID,
    ) -> Result<bool> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let vote = Vote {
            populist_user_id: uuid::Uuid::parse_str(&populist_user_id)?,
            votable_id: uuid::Uuid::parse_str(&argument_id)?,
            votable_type: VotableType::Argument,
            direction: VoteDirection::DOWN,
        };
        Vote::upsert(db_pool, vote).await?;

        Ok(true)
    }
}
