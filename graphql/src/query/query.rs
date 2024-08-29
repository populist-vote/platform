use async_graphql::{MergedObject, Object};

use super::{
    admin::AdminQuery,
    auth::AuthQuery,
    ballot_measure::BallotMeasureQuery,
    bill::BillQuery,
    candidate_guide::CandidateGuideQuery,
    election::ElectionQuery,
    embed::EmbedQuery,
    issue_tag::IssueTagQuery,
    office::OfficeQuery,
    organization::OrganizationQuery,
    politician::PoliticianQuery,
    question::{QuestionQuery, QuestionSubmissionQuery},
    race::RaceQuery,
    respondent::RespondentQuery,
    user::UserQuery,
    voting_guide::VotingGuideQuery,
};

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    /// Returns `true` to indicate the GraphQL server is reachable
    async fn health(&self) -> bool {
        true
    }
}

#[derive(MergedObject, Default)]
pub struct Query(
    AdminQuery,
    BallotMeasureQuery,
    BillQuery,
    CandidateGuideQuery,
    ElectionQuery,
    EmbedQuery,
    IssueTagQuery,
    HealthQuery,
    OfficeQuery,
    OrganizationQuery,
    PoliticianQuery,
    RaceQuery,
    RespondentQuery,
    AuthQuery,
    VotingGuideQuery,
    UserQuery,
    QuestionQuery,
    QuestionSubmissionQuery,
);
