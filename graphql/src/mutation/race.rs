use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::RaceResult};
use async_graphql::{Context, Object, Result, SimpleObject};
use db::{CreateRaceInput, Race, UpdateRaceInput};

#[derive(Default)]
pub struct RaceMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteRaceResult {
    id: String,
}

#[Object]
impl RaceMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn create_race(&self, ctx: &Context<'_>, input: CreateRaceInput) -> Result<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_race = Race::create(&db_pool, &input).await?;
        Ok(new_race.into())
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn update_race(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateRaceInput,
    ) -> Result<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let updated_race = Race::update(&db_pool, uuid::Uuid::parse_str(&id)?, &input).await?;
        Ok(updated_race.into())
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_race(&self, ctx: &Context<'_>, id: String) -> Result<DeleteRaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Race::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteRaceResult { id })
    }
}
