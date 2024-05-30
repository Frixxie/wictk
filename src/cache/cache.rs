use std::{collections::HashMap, sync::Arc};

use log::info;
use tokio::{sync::RwLock, time::Instant};

pub trait TimedCache<K, V> {
    async fn get(&self, key: K) -> Option<V>;
    async fn set(&self, key: K, value: V, expiration: Instant);
}

#[derive(Clone)]
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
    K: std::cmp::Eq + std::hash::Hash,
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
            info!("Key has expired removing");
            self.cache.write().await.remove(&key);
            return None;
        }

        info!("Returning value from cache");
        Some(value)
    }

    async fn set(&self, key: K, value: V, duration: Instant) {
        info!("Inserting into cache!");
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
