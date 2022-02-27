use crate::context::ApiContext;

use super::IssueTagResult;
use async_graphql::*;
use db::Organization;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OrganizationResult {
    id: ID,
    slug: String,
    name: String,
    description: Option<String>,
    thumbnail_image_url: Option<String>,
    website_url: Option<String>,
}

#[ComplexObject]
impl OrganizationResult {
    async fn issue_tags(&self, ctx: &Context<'_>) -> FieldResult<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records =
            Organization::issue_tags(&db_pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(IssueTagResult::from).collect();
        Ok(results)
    }
}

impl From<Organization> for OrganizationResult {
    fn from(o: Organization) -> Self {
        Self {
            id: ID::from(o.id),
            slug: o.slug,
            name: o.name,
            description: o.description,
            thumbnail_image_url: o.thumbnail_image_url,
            website_url: o.website_url,
        }
    }
}
