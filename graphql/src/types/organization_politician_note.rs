use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use chrono::{DateTime, Utc};
use db::{IssueTag, Organization, Politician};
use serde_json::Value as JSON;

use crate::context::ApiContext;

use super::{IssueTagResult, OrganizationResult, PoliticianResult};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OrganizationPoliticianNoteResult {
    pub id: ID,
    pub organization_id: ID,
    pub candidate_id: ID,
    pub election_id: ID,
    pub notes: JSON,
    pub issue_tag_ids: Vec<ID>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl OrganizationPoliticianNoteResult {
    async fn organization(&self, ctx: &Context<'_>) -> Result<OrganizationResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let organization = Organization::find_by_id(
            &db_pool,
            uuid::Uuid::parse_str(&self.organization_id).unwrap(),
        )
        .await?;
        Ok(organization.into())
    }

    async fn candidate(&self, ctx: &Context<'_>) -> Result<PoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let candidate =
            Politician::find_by_id(&db_pool, uuid::Uuid::parse_str(&self.candidate_id).unwrap())
                .await?;
        Ok(candidate.into())
    }

    async fn issue_tags(&self, ctx: &Context<'_>) -> Result<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let issue_tags = IssueTag::find_by_ids(
            &db_pool,
            self.issue_tag_ids
                .iter()
                .map(|id| uuid::Uuid::parse_str(id.as_str()).unwrap())
                .collect(),
        )
        .await?;
        Ok(issue_tags.into_iter().map(|it| it.into()).collect())
    }
}
