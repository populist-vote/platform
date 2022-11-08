use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use chrono::{DateTime, Utc};
use db::{loaders::politician::PoliticianId, IssueTag, Organization, OrganizationPoliticianNote};
use serde_json::Value as JSON;

use crate::context::ApiContext;

use super::{IssueTagResult, OrganizationResult, PoliticianResult};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OrganizationPoliticianNoteResult {
    pub id: ID,
    pub organization_id: ID,
    pub politician_id: ID,
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

    async fn politician(&self, ctx: &Context<'_>) -> Result<PoliticianResult> {
        let politician_id = uuid::Uuid::parse_str(self.politician_id.as_str()).unwrap();

        let politician = ctx
            .data::<ApiContext>()?
            .loaders
            .politician_loader
            .load_one(PoliticianId(politician_id))
            .await?
            .map(PoliticianResult::from)
            .expect("No politician found for this note.");

        Ok(politician)
    }

    async fn issue_tags(&self, ctx: &Context<'_>) -> Result<Vec<IssueTagResult>> {
        let issue_tag_ids = self
            .issue_tag_ids
            .iter()
            .map(|id| uuid::Uuid::parse_str(id.as_str()).unwrap())
            .collect::<Vec<uuid::Uuid>>();
        let issue_tags = ctx
            .data::<ApiContext>()?
            .loaders
            .issue_tag_loader
            .load_many(issue_tag_ids)
            .await?;
        let issue_tag_results = issue_tags
            .values()
            .cloned()
            .collect::<Vec<IssueTag>>()
            .into_iter()
            .map(IssueTagResult::from)
            .collect();
        Ok(issue_tag_results)
    }
}

impl From<OrganizationPoliticianNote> for OrganizationPoliticianNoteResult {
    fn from(opn: db::OrganizationPoliticianNote) -> Self {
        Self {
            id: opn.id.into(),
            organization_id: opn.organization_id.into(),
            politician_id: opn.politician_id.into(),
            election_id: opn.election_id.into(),
            notes: opn.notes,
            issue_tag_ids: opn.issue_tag_ids.into_iter().map(|id| id.into()).collect(),
            created_at: opn.created_at,
            updated_at: opn.updated_at,
        }
    }
}
