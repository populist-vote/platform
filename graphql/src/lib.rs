mod mutation;
mod query;
pub mod types;
use async_graphql::{EmptySubscription, Schema, SchemaBuilder};
use sqlx::PgPool;

use crate::mutation::Mutation;
use crate::query::Query;

pub fn new_schema(db_pool: PgPool) -> SchemaBuilder<Query, Mutation, EmptySubscription> {
    Schema::build(
        Query::default(),
        Mutation::default(),
        EmptySubscription, //Subscription::default(),
    )
    .data(db_pool)
}
