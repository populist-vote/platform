use async_graphql::{ComplexObject, Context, Enum, FieldResult, SimpleObject, ID};
use db::{models::politician::Politician, DateTime, State};
use sqlx::{Pool, Postgres};

use super::OrganizationResult;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum OfficeType {
    House,
    Senate,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PoliticianResult {
    id: ID,
    slug: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    home_state: State,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl PoliticianResult {
    async fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle_name) => format!(
                "{} {} {}",
                &self.first_name,
                middle_name.to_string(),
                &self.last_name
            ),
            None => format!("{} {}", &self.first_name, &self.last_name),
        }
    }

    async fn endorsements(&self, ctx: &Context<'_>) -> FieldResult<Vec<OrganizationResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records =
            Politician::endorsements(pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records
            .into_iter()
            .map(|r| OrganizationResult::from(r))
            .collect();
        Ok(results)
    }
}

impl From<Politician> for PoliticianResult {
    fn from(p: Politician) -> Self {
        Self {
            id: ID::from(p.id),
            slug: p.slug,
            first_name: p.first_name,
            middle_name: p.middle_name,
            last_name: p.last_name,
            home_state: p.home_state,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
