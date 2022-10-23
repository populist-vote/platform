use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct Heartbeat {
    utc: DateTime<Utc>,
}

impl Heartbeat {
    pub fn new() -> Self {
        Heartbeat { utc: Utc::now() }
    }
}
