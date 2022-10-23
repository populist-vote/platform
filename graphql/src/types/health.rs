use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct Heartbeat {
    utc: DateTime<Utc>,
}

impl Heartbeat {
    fn new() -> Self {
        Heartbeat { utc: Utc::now() }
    }
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self::new()
    }
}
