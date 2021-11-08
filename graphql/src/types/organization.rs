use async_graphql::*;
use db::Organization;

#[derive(SimpleObject)]
pub struct OrganizationResult {
    id: ID,
    slug: String,
    name: String,
    description: Option<String>,
    thumbnail_image_url: Option<String>,
    website_url: Option<String>,
}

// Why cant this just automatically happen?
impl From<Organization> for OrganizationResult {
    fn from(o: Organization) -> Self {
        Self {
            id: ID::from(o.id),
            slug: o.slug,
            name: o.name,
            description: o.description,
            thumbnail_image_url: o.thumbnail_image_url,
            website_url: o.website_url
        }
    }
}