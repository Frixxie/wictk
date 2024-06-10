use std::marker::PhantomData;

use redis::{Client, Commands, FromRedisValue, JsonCommands, ToRedisArgs};

use crate::handlers::Alerts;

use super::TimedCache;

#[derive(Clone, Debug)]
pub struct RemoteCache<K, V> {
    client: Client,
    key: PhantomData<K>,
    value: PhantomData<V>,
}

impl<K, V> RemoteCache<K, V> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            key: PhantomData,
            value: PhantomData,
        }
    }

    pub fn get_alert(&self, key: &str) -> Option<Alerts> {
        let mut connection = self.client.get_connection().unwrap();
        let value_str: String = connection.json_get(key.to_string(), "").unwrap();
        let value: Alerts = serde_json::from_str(&value_str).unwrap();
        Some(value)
    }

    pub fn set_alert(&self, key: &str, value: Alerts, expiration: tokio::time::Instant) {
        let mut connection = self.client.get_connection().unwrap();
        let expiration = expiration.elapsed().as_secs();
        let value_str: String = serde_json::to_string(&value).unwrap();
        let key_str = key.to_string();
        let _old: String = connection.json_set(key_str, "", &value_str).unwrap();
    }
}

impl<K, V> TimedCache<K, V> for RemoteCache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + std::fmt::Display + ToRedisArgs,
    V: Clone + FromRedisValue + ToRedisArgs,
{
    async fn get(&self, key: K) -> Option<V> {
        let mut connection = self.client.get_connection().unwrap();
        let value: V = connection.get(key).unwrap();
        Some(value)
    }

    async fn set(&self, key: K, value: V, expiration: tokio::time::Instant) {
        let mut connection = self.client.get_connection().unwrap();
        let expiration = expiration.elapsed().as_secs();
        let _old: V = connection.set(key, value).unwrap();
    }
}
