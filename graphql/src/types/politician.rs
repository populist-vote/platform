use async_graphql::{ComplexObject, Enum, SimpleObject, ID};
use db::models::politician::Politician;

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
    home_state: String,
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
}

// Why cant this just automatically happen?
impl From<Politician> for PoliticianResult {
    fn from(p: Politician) -> Self {
        Self {
            id: ID::from(p.id),
            slug: p.slug,
            first_name: p.first_name,
            middle_name: p.middle_name,
            last_name: p.last_name,
            home_state: p.home_state,
        }
    }
}
