use async_graphql::{Context, Object, Schema};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Cache<K, V> {
    store: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> Cache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Cache {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    pub fn set(&self, key: K, value: V) {
        let mut store = self.store.write().unwrap();
        store.insert(key, value);
    }
}
