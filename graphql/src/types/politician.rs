use async_graphql::*;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum OfficeType {
    House,
    Senate,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Politician {
    id: ID,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
}

#[ComplexObject]
impl Politician {
    async fn full_name(&self) -> String {
        format!(
            "{} {:?} {}",
            &self.first_name, &self.middle_name, &self.last_name
        )
    }
}