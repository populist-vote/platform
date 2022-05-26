use async_graphql::{MergedObject, Object};

use super::{
    auth::AuthQuery, ballot_measure::BallotMeasureQuery, bill::BillQuery, election::ElectionQuery,
    issue_tag::IssueTagQuery, office::OfficeQuery, organization::OrganizationQuery,
    politician::PoliticianQuery, race::RaceQuery, voting_guide::VotingGuideQuery,
};

#[derive(Default)]
pub struct MainQuery;

#[Object]
impl MainQuery {
    async fn health(&self) -> bool {
        true
    }
}

#[derive(MergedObject, Default)]
pub struct Query(
    BallotMeasureQuery,
    BillQuery,
    ElectionQuery,
    IssueTagQuery,
    MainQuery,
    OfficeQuery,
    OrganizationQuery,
    PoliticianQuery,
    RaceQuery,
    AuthQuery,
    VotingGuideQuery,
);
