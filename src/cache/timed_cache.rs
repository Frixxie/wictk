use std::{collections::HashMap, sync::Arc};

use metrics::gauge;
use tokio::{sync::RwLock, time::Instant};
use tracing::info;

pub trait TimedCache<K, V> {
    async fn get(&self, key: K) -> Option<V>;
    async fn set(&self, key: K, value: V, expiration: Instant);
}

#[derive(Clone, Debug)]
pub struct Cache<K, V> {
    cache: Arc<RwLock<HashMap<K, (Instant, V)>>>,
}

impl<K, V> Cache<K, V> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K, V> Default for Cache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> TimedCache<K, V> for Cache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + std::fmt::Display,
    V: Clone,
{
    async fn get(&self, key: K) -> Option<V> {
        let (duration, value) = {
            let cache = self.cache.read().await;
            match cache.get(&key) {
                Some((duration, value)) => (*duration, value.clone()),
                None => return None,
            }
        };

        if Instant::now() > duration {
            info!("Key {key} has expired removing");
            self.cache.write().await.remove(&key);
            gauge!("cache_size").decrement(1);
            return None;
        }

        info!("Returning {key}'s value from cache");
        Some(value)
    }

    async fn set(&self, key: K, value: V, duration: Instant) {
        gauge!("cache_size").increment(1);
        info!("Inserting {} into cache!", key);
        self.cache.write().await.insert(key, (duration, value));
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache() {
        let cache = Cache::new();
        let key = "key";
        let value = "value";

        let duration = Instant::now() + Duration::from_secs(1);

        cache.set(key, value, duration).await;

        assert_eq!(cache.get(key).await, Some(value));

        std::thread::sleep(Duration::from_secs(2));

        assert_eq!(cache.get(key).await, None);
    }
}
