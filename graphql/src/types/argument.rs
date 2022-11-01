use async_graphql::{ComplexObject, Context, FieldResult, Result, SimpleObject, Union, ID};
use db::{
    models::{enums::AuthorType, vote::Vote},
    Argument, Organization, Politician,
};

use crate::context::ApiContext;

use super::{OrganizationResult, PoliticianResult};

#[derive(Debug, Clone, Union)]
enum AuthorResult {
    PoliticianResult(Box<PoliticianResult>),
    OrganizationResult(Box<OrganizationResult>),
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
}

#[ComplexObject]
impl ArgumentResult {
    async fn author(&self, ctx: &Context<'_>) -> FieldResult<AuthorResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let result = match self.author_type {
            AuthorType::Politician => {
                AuthorResult::PoliticianResult(Box::new(PoliticianResult::from(
                    Politician::find_by_id(
                        &db_pool,
                        uuid::Uuid::parse_str(self.author_id.as_str())?,
                    )
                    .await?,
                )))
            }
            AuthorType::Organization => {
                AuthorResult::OrganizationResult(Box::new(OrganizationResult::from(
                    Organization::find_by_id(
                        &db_pool,
                        uuid::Uuid::parse_str(self.author_id.as_str())?,
                    )
                    .await?,
                )))
            }
        };

        Ok(result)
    }

    async fn votes(&self, ctx: &Context<'_>) -> Result<i64> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let total = Vote::count(&db_pool, uuid::Uuid::parse_str(self.id.as_str())?).await?;
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
        }
    }
}
