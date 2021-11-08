use async_graphql::{MergedObject};
use super::{organization::OrganizationMutation, politician::PoliticianMutation};
#[derive(MergedObject, Default)]
pub struct Mutation(PoliticianMutation, OrganizationMutation);
