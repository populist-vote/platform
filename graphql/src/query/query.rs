use async_graphql::{Result, Error, Object};

use crate::types::{Organization, Politician};

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn health(&self) -> bool {
        true
    }

    async fn politicians(&self, state: String) -> Result<Vec<Politician>> {
        // Look up politician by state, office, etc, in the database
        Ok(vec![])
    }

    async fn politician_by_id(&self, id: String) -> Result<Politician> {
        // Look up politician by id in the database
        todo!()
    }

    async fn politician_by_name(&self, query: String) -> Result<Option<Politician>, Error> {
        // Fuzzy search for politician by full name
        todo!()
    }

    async fn organizations(&self, state: String) -> Result<Vec<Organization>> {
        Ok(vec![])
    }
}