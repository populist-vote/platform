use crate::{context::ApiContext, guard::StaffOnly, is_admin, types::BillResult, SessionData};
use async_graphql::*;
use auth::AccessTokenClaims;
use db::{
    models::enums::ArgumentPosition, Bill, CreateArgumentInput, PublicVotes, UpsertBillInput,
};
use jsonwebtoken::TokenData;
use sqlx::{Pool, Postgres};
#[derive(Default)]
pub struct BillMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeleteBillResult {
    id: String,
}

#[derive(SimpleObject)]
struct UpsertBillPublicVoteResult {
    bill_id: String,
    position: Option<ArgumentPosition>,
    public_votes: PublicVotes,
}

async fn handle_nested_arguments(
    db_pool: &Pool<Postgres>,
    bill_id: uuid::Uuid,
    arguments_input: Vec<CreateArgumentInput>,
) -> Result<(), Error> {
    if !arguments_input.is_empty() {
        for input in arguments_input {
            Bill::create_bill_argument(
                db_pool,
                bill_id,
                uuid::Uuid::parse_str(&input.author_id)?,
                &input,
            )
            .await?;
        }
    }
    Ok(())
}

#[Object]
impl BillMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn upsert_bill(&self, ctx: &Context<'_>, input: UpsertBillInput) -> Result<BillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Bill::upsert(&db_pool, &input).await?;
        if input.arguments.is_some() {
            handle_nested_arguments(&db_pool, new_record.id, input.arguments.unwrap()).await?;
        }
        Ok(BillResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_bill(&self, ctx: &Context<'_>, id: String) -> Result<DeleteBillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Bill::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeleteBillResult { id })
    }

    #[graphql(visible = "is_admin")]
    async fn upsert_bill_public_vote(
        &self,
        ctx: &Context<'_>,
        bill_id: ID,
        position: Option<ArgumentPosition>,
    ) -> Result<UpsertBillPublicVoteResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user = ctx.data::<Option<TokenData<AccessTokenClaims>>>()?;
        let user_id = user.as_ref().map(|u| u.claims.sub.to_string());
        let session_data = ctx.data::<SessionData>()?.clone();
        let session_id = session_data.session_id;

        let public_votes = Bill::upsert_public_vote(
            &db_pool,
            uuid::Uuid::parse_str(&bill_id)?,
            user_id.map(|id| uuid::Uuid::parse_str(&id)).transpose()?,
            Some(uuid::Uuid::parse_str(&session_id.to_string())?),
            position,
        )
        .await?;
        Ok(UpsertBillPublicVoteResult {
            bill_id: bill_id.to_string(),
            position,
            public_votes,
        })
    }
}
