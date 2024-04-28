use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::Coordinates;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenWeatherMapLocation {
    pub name: String,
    pub local_names: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub location: Coordinates,
    pub country: String,
    pub state: Option<String>,
}

impl OpenWeatherMapLocation {
    pub async fn fetch(client: &Client, location: &str) -> Option<Vec<Self>> {
        match client
            .get("https://api.openweathermap.org/geo/1.0/direct")
            .query(&[("q", location)])
            .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
            .send()
            .await
        {
            Ok(result) => {
                log::info!("Statuscode from openweathermap: {}", result.status());
                match result.json::<Vec<OpenWeatherMapLocation>>().await {
                    Ok(res) => Some(res),
                    Err(err) => {
                        log::error!("Error: {}", err);
                        None
                    }
                }
            }
            Err(err) => {
                log::error!("Error: {}", err);
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationError {
    pub message: String,
}

impl LocationError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for LocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LocationError: {}", self.message)
    }
}

impl std::error::Error for LocationError {}
