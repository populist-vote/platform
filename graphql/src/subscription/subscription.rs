use async_graphql::MergedSubscription;

use super::HealthSubscription;

#[derive(MergedSubscription, Default)]
pub struct Subscription(HealthSubscription);
