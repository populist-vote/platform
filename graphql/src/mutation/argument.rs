use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::ArgumentResult};
use async_graphql::*;
use db::{
    models::vote::{VotableType, Vote, VoteDirection},
    Argument, UpdateArgumentInput,
};

#[derive(Default)]
pub struct ArgumentMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteArgumentResult {
    id: String,
}

#[Object]
impl ArgumentMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn update_argument(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateArgumentInput,
    ) -> Result<ArgumentResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_record =
            Argument::update(&db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(ArgumentResult::from(updated_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_argument(&self, ctx: &Context<'_>, id: String) -> Result<DeleteArgumentResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Argument::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteArgumentResult { id })
    }

    #[graphql(visible = "is_admin")]
    async fn upvote_argument(
        &self,
        ctx: &Context<'_>,
        argument_id: ID,
        populist_user_id: ID,
    ) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let vote = Vote {
            populist_user_id: uuid::Uuid::parse_str(&populist_user_id)?,
            votable_id: uuid::Uuid::parse_str(&argument_id)?,
            votable_type: VotableType::Argument,
            direction: VoteDirection::UP,
        };
        Vote::upsert(&db_pool, vote).await?;

        Ok(true)
    }

    #[graphql(visible = "is_admin")]
    async fn downvote_argument(
        &self,
        ctx: &Context<'_>,
        argument_id: ID,
        populist_user_id: ID,
    ) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let vote = Vote {
            populist_user_id: uuid::Uuid::parse_str(&populist_user_id)?,
            votable_id: uuid::Uuid::parse_str(&argument_id)?,
            votable_type: VotableType::Argument,
            direction: VoteDirection::DOWN,
        };
        Vote::upsert(&db_pool, vote).await?;

        Ok(true)
    }
}
