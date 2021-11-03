mod mutation;
mod query;
pub mod types;
use async_graphql::{EmptyMutation, EmptySubscription, Schema, SchemaBuilder};

use crate::query::Query;

pub fn new_schema() -> SchemaBuilder<Query, EmptyMutation, EmptySubscription> {
    Schema::build(
        Query::default(),
        EmptyMutation,
        EmptySubscription, //Subscription::default(),
    )
}
