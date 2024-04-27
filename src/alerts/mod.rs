mod met;

pub use met::MetAlert;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::handlers::location::Coordinates;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Alert {
    /// The alert was issued by the National Weather Service.
    Met(MetAlert),
    /// The alert was issued by a local authority, typically a county.
    Nve,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    /// The alert is for a moderate event.
    Yellow,
    /// The alert is for a severe event.
    Orange,
    /// The alert is for an extreme event.
    Red,
}

#[derive(Debug)]
pub struct AlertError {
    pub message: String,
}

impl std::fmt::Display for AlertError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AlertError: {}", self.message)
    }
}

impl std::error::Error for AlertError {}

impl AlertError {
    pub fn new(message: &str) -> Self {
        AlertError {
            message: message.to_owned(),
        }
    }
}

pub trait AlertFetcher {
    async fn fetch(client: Client, location: Coordinates) -> Result<Vec<Alert>, AlertError>;
}
