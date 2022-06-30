use async_graphql::{Context, Object};

use crate::{context::ApiContext, guard::StaffOnly, types::Error};

#[derive(Default)]
pub struct AdminQuery;

#[Object]
impl AdminQuery {
    /// Get all users
    #[graphql(guard = "StaffOnly")]
    async fn user_count(&self, ctx: &Context<'_>) -> Result<Option<i64>, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        let user_count_record = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM populist_user
        "#
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(user_count_record.count)
    }
}
