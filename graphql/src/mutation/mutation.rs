use super::{
    ballot_measure::BallotMeasureMutation, bill::BillMutation, election::ElectionMutation,
    issue_tag::IssueTagMutation, organization::OrganizationMutation,
    politician::PoliticianMutation,
};
use async_graphql::MergedObject;
#[derive(MergedObject, Default)]
pub struct Mutation(
    PoliticianMutation,
    OrganizationMutation,
    BillMutation,
    BallotMeasureMutation,
    ElectionMutation,
    IssueTagMutation,
);
