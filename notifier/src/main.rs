use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use wictk_core::{Alert, MetAlert};

use crate::notifications::{Notifyer, NtfyNotifyer};
mod alerts;
mod notifications;

pub async fn get_met_alerts(client: &Client, url: &str) -> Result<Vec<Alert>> {
    let resp = client.get(url).send().await?;
    let alerts: Vec<Alert> = resp.json().await?;
    Ok(alerts)
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    let mut alerter = NtfyNotifyer::new(client.clone(), "https://ntfy.frikk.io".to_string());

    loop {
        let alerts = get_met_alerts(&client, "https://wictk.frikk.io/api/alerts").await?;

        println!("Fetched {} alerts", alerts.len());

        for alert in alerts {
            match alert {
                Alert::Met(alert) => {
                    let notification: crate::notifications::Notification = alert.into();
                    match alerter.publish(notification, "weather_alerts").await {
                        Some(_) => {
                            println!("Notification sent");
                        }
                        None => {
                            println!("Notification was not sent");
                        }
                    }
                }
                _ => println!("Unknown alert type"),
            }
        }

        let alerts = alerter
            .get_notifications(&"weather_alerts".to_string())
            .await;

        match alerts {
            Some(alerts) => {
                for alert in alerts {
                    println!("Sent notification: {:?}", alert);
                }
            }
            None => {
                println!("No notifications sent yet");
            }
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
