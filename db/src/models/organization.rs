use crate::models::{issue_tag::IssueTag, user::User};
use crate::{DateTime, Id};

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Organization {
    pub id: Id,
    pub name: String,
    pub thumbnail_image_url: Option<String>,
    pub description: Option<String>,
    pub website_url: Option<String>,
    pub issue_tags: Vec<IssueTag>,
    pub created_by: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// impl Organization {
// pub async fn get_by_id() -> Result<Self, Error> {

// }
// }
