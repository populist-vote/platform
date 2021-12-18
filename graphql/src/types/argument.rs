use async_graphql::{ComplexObject, Context, FieldResult, Result, SimpleObject, Union, ID};
use db::{
    models::{enums::AuthorType, vote::Vote},
    Argument, DateTime, Organization, Politician,
};
use sqlx::{Pool, Postgres};

use super::{OrganizationResult, PoliticianResult};

#[derive(Debug, Clone, Union)]
enum AuthorResult {
    PoliticianResult(PoliticianResult),
    OrganizationResult(OrganizationResult),
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ArgumentResult {
    id: ID,
    author_id: ID,
    author_type: AuthorType,
    title: String,
    position: String,
    body: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl ArgumentResult {
    async fn author(&self, ctx: &Context<'_>) -> FieldResult<AuthorResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let result = match self.author_type {
            AuthorType::Politician => AuthorResult::PoliticianResult(PoliticianResult::from(
                Politician::find_by_id(pool, uuid::Uuid::parse_str(self.author_id.as_str())?)
                    .await?,
            )),
            AuthorType::Organization => AuthorResult::OrganizationResult(OrganizationResult::from(
                Organization::find_by_id(pool, uuid::Uuid::parse_str(self.author_id.as_str())?)
                    .await?,
            )),
        };

        Ok(result)
    }

    async fn votes(&self, ctx: &Context<'_>) -> Result<i64> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let total = Vote::count(pool, uuid::Uuid::parse_str(self.id.as_str())?).await?;
        Ok(total)
    }
}

impl From<Argument> for ArgumentResult {
    fn from(a: Argument) -> Self {
        Self {
            id: ID::from(a.id),
            author_id: ID::from(a.author_id),
            author_type: a.author_type,
            title: a.title,
            body: a.body,
            position: a.position.to_string(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}
