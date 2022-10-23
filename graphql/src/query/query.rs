use async_graphql::{MergedObject, Object};

use super::{
    admin::AdminQuery, auth::AuthQuery, ballot_measure::BallotMeasureQuery, bill::BillQuery,
    election::ElectionQuery, issue_tag::IssueTagQuery, office::OfficeQuery,
    organization::OrganizationQuery, politician::PoliticianQuery, race::RaceQuery, user::UserQuery,
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
    ElectionQuery,
    IssueTagQuery,
    HealthQuery,
    OfficeQuery,
    OrganizationQuery,
    PoliticianQuery,
    RaceQuery,
    AuthQuery,
    VotingGuideQuery,
    UserQuery,
);
