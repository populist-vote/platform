use super::{
    argument::ArgumentMutation, ballot_measure::BallotMeasureMutation, bill::BillMutation,
    election::ElectionMutation, issue_tag::IssueTagMutation, organization::OrganizationMutation,
    politician::PoliticianMutation, user::UserMutation,
};
use async_graphql::MergedObject;
#[derive(MergedObject, Default)]
pub struct Mutation(
    ArgumentMutation,
    PoliticianMutation,
    OrganizationMutation,
    BillMutation,
    BallotMeasureMutation,
    ElectionMutation,
    IssueTagMutation,
    UserMutation,
);
