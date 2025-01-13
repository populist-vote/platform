use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;

pub struct Cache<K, V> {
    store: Arc<RwLock<HashMap<K, (V, Instant)>>>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(ttl: Duration) -> Self {
        let cache = Cache {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        };

        let store_clone = Arc::clone(&cache.store);
        let ttl_clone = cache.ttl;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut store = store_clone.write().unwrap();
                let now = Instant::now();
                store.retain(|_, &mut (_, timestamp)| now.duration_since(timestamp) < ttl_clone);
            }
        });

        cache
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let store = self.store.read().unwrap();
        store.get(key).map(|(value, _)| value.clone())
    }

    pub fn set(&self, key: K, value: V) {
        let mut store = self.store.write().unwrap();
        store.insert(key, (value, Instant::now()));
    }

    pub fn invalidate(&self, key: &K) {
        let mut store = self.store.write().unwrap();
        store.remove(key);
    }
}
