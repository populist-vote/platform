use super::{
    bill::BillMutation, organization::OrganizationMutation, politician::PoliticianMutation,
};
use async_graphql::MergedObject;
#[derive(MergedObject, Default)]
pub struct Mutation(PoliticianMutation, OrganizationMutation, BillMutation);
