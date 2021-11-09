use async_graphql::{MergedObject, Object};

use super::{bill::BillQuery, organization::OrganizationQuery, politician::PoliticianQuery};

#[derive(Default)]
pub struct MainQuery;

#[Object]
impl MainQuery {
    async fn health(&self) -> bool {
        true
    }
}

#[derive(MergedObject, Default)]
pub struct Query(MainQuery, PoliticianQuery, OrganizationQuery, BillQuery);
