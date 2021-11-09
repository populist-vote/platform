use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::{
    models::{bill::Bill, legislation::LegislationStatus},
    DateTime,
};
use sqlx::{Pool, Postgres};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BillResult {
    id: ID,
    slug: String,
    name: String,
    vote_status: LegislationStatus,
    description: Option<String>,
    official_summary: Option<String>,
    populist_summary: Option<String>,
    full_text_url: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl BillResult {
    async fn arguments(&self, ctx: &Context<'_>) -> FieldResult<Vec<BillResult>> {
        //Change to ArgumentResult once implemented
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        todo!()
    }
}

impl From<Bill> for BillResult {
    fn from(b: Bill) -> Self {
        Self {
            id: ID::from(b.id),
            slug: b.slug,
            name: b.name,
            vote_status: b.vote_status,
            description: b.description,
            official_summary: b.official_summary,
            populist_summary: b.populist_summary,
            full_text_url: b.full_text_url,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}
