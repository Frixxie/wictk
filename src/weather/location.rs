use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenWeatherLocationEntry {
    pub name: String,
    pub local_names: HashMap<String, String>,
    #[serde(flatten)]
    pub location: Location,
    pub country: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    pub lat: f32,
    pub lon: f32,
}

impl Location {
    pub fn new(lat: f32, lon: f32) -> Self {
        Self { lat, lon }
    }
}
