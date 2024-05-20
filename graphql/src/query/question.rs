use crate::context::ApiContext;
use crate::types::QuestionResult;
use async_graphql::{Context, Object, Result, ID};
use db::Question;

#[derive(Default)]
pub struct QuestionQuery;

#[Object]
impl QuestionQuery {
    async fn question_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<QuestionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = Question::find_by_id(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(record.into())
    }
}
