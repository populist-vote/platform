use async_graphql::{Context, InputObject, Object};
use db::models::enums::State;

use crate::{context::ApiContext, guard::StaffOnly, types::Error};

#[derive(Default)]
pub struct AdminQuery;

#[derive(Default, InputObject)]
struct UserCountFilter {
    state: Option<State>,
}

#[Object]
impl AdminQuery {
    /// Get all users
    #[graphql(guard = "StaffOnly")]
    async fn user_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<UserCountFilter>,
    ) -> Result<Option<i64>, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();

        let filter = match filter {
            Some(filter) => filter,
            None => UserCountFilter::default(),
        };

        let user_count_record = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM populist_user
            JOIN user_profile ON populist_user.id = user_profile.user_id
            JOIN address a ON user_profile.address_id = a.id
            WHERE ($1::text IS NULL OR a.state = $1)
        "#,
            filter.state.map(|s| s.to_string())
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(user_count_record.count)
    }
}
