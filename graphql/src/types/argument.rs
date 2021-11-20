use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, Union, ID};
use db::{models::argument::Argument, DateTime};
use sqlx::{Pool, Postgres};

use super::{OrganizationResult, PoliticianResult};

#[derive(Union)]
enum AuthorResult {
    Politician(PoliticianResult),
    Organization(OrganizationResult),
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ArgumentResult {
    id: ID,
    title: String,
    position: String,
    body: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl ArgumentResult {
    async fn author(&self, ctx: &Context<'_>) -> FieldResult<AuthorResult> {
        let _pool = ctx.data_unchecked::<Pool<Postgres>>();
        todo!()
    }
}

impl From<Argument> for ArgumentResult {
    fn from(a: Argument) -> Self {
        Self {
            id: ID::from(a.id),
            title: a.title,
            body: a.body,
            position: a.position.to_string(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}
