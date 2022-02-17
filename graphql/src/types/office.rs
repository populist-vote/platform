use crate::types::PoliticianResult;
use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::{
    models::{
        enums::{PoliticalScope, State},
        office::Office,
    },
    DateTime,
};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OfficeResult {
    id: ID,
    slug: String,
    title: String,
    political_scope: PoliticalScope,
    state: Option<State>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl OfficeResult {
    async fn encumbent(&self, _ctx: &Context<'_>) -> FieldResult<PoliticianResult> {
        todo!()
    }
}

impl From<Office> for OfficeResult {
    fn from(o: Office) -> Self {
        Self {
            id: ID::from(o.id),
            slug: o.slug,
            title: o.title,
            political_scope: o.political_scope,
            state: o.state,
            created_at: o.created_at,
            updated_at: o.updated_at,
        }
    }
}
