use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::RaceResult};
use async_graphql::{Context, Object, Result, SimpleObject};
use db::{Race, UpsertRaceInput};

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
    async fn upsert_race(&self, ctx: &Context<'_>, input: UpsertRaceInput) -> Result<RaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_race = Race::upsert(&db_pool, &input).await?;
        Ok(new_race.into())
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_race(&self, ctx: &Context<'_>, id: String) -> Result<DeleteRaceResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Race::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteRaceResult { id })
    }
}
