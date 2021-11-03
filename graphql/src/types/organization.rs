
use async_graphql::*;

#[derive(SimpleObject)]
pub struct IssueTag {
  id: ID, 
  name: String,
  description: String,
}

#[derive(SimpleObject)]
pub struct Organization {
    id: ID,
    name: String,
    thumbnail_image_url: String,
    description: String,
    website_url: String,
    organization_type_id: String,
    issue_tags: Vec<IssueTag>,
}