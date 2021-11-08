use crate::models::{organization::Organization, user::User};
use crate::DateTime;
use sqlx::Error;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct IssueTag {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub politicians: Vec<uuid::Uuid>,
    pub organizations: Vec<Organization>,
    pub created_by: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl IssueTag {
    pub async fn new(ctx: (), name: &str, description: &str) -> Result<Self, Error> {
        let id = uuid::Uuid::new_v4();
        todo!()
        // let mut conn = ctx.pool.acquire().await?;
        // let mut tx = conn.begin().await?;

        // let query = sqlx::query!(
        //     "INSERT INTO issue_tag (id, name, description) VALUES ($1, $2, $3)",
        //     id,
        //     name,
        //     description
        // )
        // .execute(ctx)
        // .await?;

        // let created_issue_tag = query.fetch_one(&mut tx).await?;

        // tx.commit().await?;

        // Ok(created_issue_tag)
    }
}
