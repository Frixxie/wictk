use std::error::Error;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::location::Coordinates;

use super::{MetNowcast, OpenWeatherNowcast};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Nowcast {
    Met(MetNowcast),
    OpenWeather(OpenWeatherNowcast),
}

#[async_trait]
pub trait NowcastFetcher {
    async fn fetch(client: Client, location: Coordinates) -> Result<Nowcast, NowcastError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowcastError {
    pub message: String,
}

impl NowcastError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for NowcastError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NowcastError: {}", self.message)
    }
}

impl Error for NowcastError {}
