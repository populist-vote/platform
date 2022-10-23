use async_graphql::Subscription;
use tokio::time::Duration;
use tokio_stream::{wrappers::IntervalStream, Stream, StreamExt};

use crate::types::Heartbeat;

#[derive(Default)]
pub struct HealthSubscription;

#[Subscription]
impl HealthSubscription {
    /// Heartbeat, containing the UTC timestamp of the last server-sent payload
    async fn heartbeat(
        &self,
        #[graphql(default = 1000, validator(minimum = 10, maximum = 60_000))] interval: i32,
    ) -> impl Stream<Item = Heartbeat> {
        tracing::info!(
            "Starting heartbeat subscription with interval {}ms",
            interval
        );
        IntervalStream::new(tokio::time::interval(Duration::from_millis(
            interval as u64,
        )))
        .map(|_| Heartbeat::default())
    }
}
