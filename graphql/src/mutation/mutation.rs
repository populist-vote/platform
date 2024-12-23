use super::{
    argument::ArgumentMutation,
    auth::AuthMutation,
    ballot_measure::BallotMeasureMutation,
    bill::BillMutation,
    candidate_guide::CandidateGuideMutation,
    conversation::ConversationMutation,
    election::ElectionMutation,
    embed::EmbedMutation,
    issue_tag::IssueTagMutation,
    office::OfficeMutation,
    organization::OrganizationMutation,
    politician::PoliticianMutation,
    poll::PollMutation,
    question::{QuestionMutation, QuestionSubmissionMutation},
    race::RaceMutation,
    user::UserMutation,
    voting_guide::VotingGuideMutation,
};
use crate::is_admin;
use async_graphql::MergedObject;
#[derive(MergedObject, Default)]
// Hide all mutations from public API
#[graphql(visible = "is_admin")]
pub struct Mutation(
    ArgumentMutation,
    ConversationMutation,
    PoliticianMutation,
    OrganizationMutation,
    BillMutation,
    BallotMeasureMutation,
    CandidateGuideMutation,
    ElectionMutation,
    EmbedMutation,
    IssueTagMutation,
    AuthMutation,
    OfficeMutation,
    RaceMutation,
    VotingGuideMutation,
    UserMutation,
    PollMutation,
    QuestionMutation,
    QuestionSubmissionMutation,
);
