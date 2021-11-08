use async_graphql::*;
use db::{CreateOrganizationInput, CreatePoliticianInput, Organization, Politician, UpdatePoliticianInput};
use sqlx::{Pool, Postgres};

use crate::types::{OrganizationResult, PoliticianResult};
#[derive(Default)]
pub struct OrganizationMutation;

#[Object]
impl OrganizationMutation {

    async fn create_organization(&self, ctx: &Context<'_>, input: CreateOrganizationInput) -> Result<OrganizationResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let new_record = Organization::create(db_pool, &input).await?;
        Ok(OrganizationResult::from(new_record))
    }
}
