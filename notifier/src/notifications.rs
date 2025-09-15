use std::collections::{HashMap, HashSet};
use std::fmt;

use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Priority {
    Urgent,
    High,
    Default,
    Low,
    Min,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Tag {
    Wind,
    Warning,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Wind => write!(f, "dash"),
            Tag::Warning => write!(f, "warning"),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Notification {
    title: String,
    priority: Priority,
    tag: Tag,
    description: String,
}

impl Notification {
    pub fn new(title: String, priority: Priority, tag: Tag, description: String) -> Self {
        Self {
            title,
            priority,
            tag,
            description,
        }
    }
}

#[derive(Deserialize)]
pub struct NotificationResponse {
    id: String,
    time: u32,
    expires: u32,
    event: String,
    topic: String,
    message: String,
}

impl NotificationResponse {
    pub fn new(
        id: String,
        time: u32,
        expires: u32,
        event: String,
        topic: String,
        message: String,
    ) -> Self {
        Self {
            id,
            time,
            expires,
            event,
            topic,
            message,
        }
    }
}

pub trait Notifyer {
    async fn publish(&mut self, alert: Notification, topic: &str) -> Option<NotificationResponse>;

    async fn get_notifications(&self, topic: &str) -> Option<Vec<Notification>>;

    async fn has_sent_alert(&self, alert: &Notification, topic: &str) -> bool {
        let notifications = self.get_notifications(topic).await;
        if let Some(notifications) = notifications {
            notifications.contains(alert)
        } else {
            false
        }
    }
}

impl Notifyer for NtfyNotifyer {
    async fn publish(&mut self, alert: Notification, topic: &str) -> Option<NotificationResponse> {
        if self.has_sent_alert(&alert, topic).await {
            return None;
        }
        let alrt_clone = alert.clone();
        let priority = match alert.priority {
            Priority::Urgent => "urgent",
            Priority::High => "high",
            Priority::Default => "default",
            Priority::Low => "low",
            Priority::Min => "min",
        };

        let url = format!("{}/{}", self.ntfy_url, topic);
        let resp = self
            .client
            .post(&url)
            .header("Title", alert.title)
            .header("Priority", priority)
            .header("Tags", alert.tag.to_string())
            .body(alert.description)
            .send()
            .await;

        match resp {
            Ok(response) => {
                if response.status().is_success() {
                    let alert_resp: NotificationResponse = response.json().await.unwrap();
                    self.add_alert(alrt_clone, topic);
                    Some(alert_resp)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    async fn get_notifications(&self, topic: &str) -> Option<Vec<Notification>> {
        if let Some(notifications) = self.sent_notifications.get(topic) {
            Some(notifications.iter().cloned().collect())
        } else {
            None
        }
    }
}

pub struct NtfyNotifyer {
    sent_notifications: HashMap<String, HashSet<Notification>>,
    client: Client,
    ntfy_url: String,
}

impl NtfyNotifyer {
    pub fn new(client: Client, ntfy_url: String) -> Self {
        Self {
            sent_notifications: HashMap::new(),
            client,
            ntfy_url,
        }
    }

    pub fn add_alert(&mut self, alert: Notification, topic: &str) {
        self.sent_notifications
            .entry(topic.to_string())
            .or_insert_with(HashSet::new)
            .insert(alert);
    }
}

#[cfg(test)]

mod tests {
    use crate::notifications::{Notifyer, NtfyNotifyer};

    #[tokio::test]
    async fn should_publish_notification_ok() {
        let client = reqwest::Client::new();
        let mut alerter = NtfyNotifyer::new(client, "https://ntfy.frikk.io".to_string());
        let alert = crate::notifications::Notification::new(
            "Test Alert".to_string(),
            crate::notifications::Priority::High,
            crate::notifications::Tag::Warning,
            "This is a test alert".to_string(),
        );
        let topic = "test_topic".to_string();
        let response = alerter.publish(alert, &topic).await;
        assert!(response.is_some());
    }

    #[tokio::test]
    async fn should_get_no_notifications() {
        let client = reqwest::Client::new();
        let alerter = NtfyNotifyer::new(client, "https://ntfy.frikk.io".to_string());
        let topic = "non_existent_topic".to_string();
        let notifications = alerter.get_notifications(&topic).await;
        assert!(notifications.is_none());
    }

    #[tokio::test]
    async fn should_add_and_get_notifications() {
        let client = reqwest::Client::new();
        let mut alerter = NtfyNotifyer::new(client, "https://ntfy.frikk.io".to_string());
        let alert = crate::notifications::Notification::new(
            "Test Alert".to_string(),
            crate::notifications::Priority::High,
            crate::notifications::Tag::Warning,
            "This is a test alert".to_string(),
        );
        let topic = "test_topic".to_string();
        alerter.add_alert(alert.clone(), &topic);
        let notifications = alerter.get_notifications(&topic).await;
        assert!(notifications.is_some());
        assert!(notifications.unwrap().contains(&alert));
    }

    #[tokio::test]
    async fn should_check_sent_alert() {
        let client = reqwest::Client::new();
        let mut alerter = NtfyNotifyer::new(client, "https://ntfy.frikk.io".to_string());
        let alert = crate::notifications::Notification::new(
            "Test Alert".to_string(),
            crate::notifications::Priority::High,
            crate::notifications::Tag::Warning,
            "This is a test alert".to_string(),
        );
        let topic = "test_topic".to_string();
        alerter.add_alert(alert.clone(), &topic);
        let has_sent = alerter.has_sent_alert(&alert, &topic).await;
        assert!(has_sent);
    }
}
