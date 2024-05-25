use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::location::LocationError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coordinates {
    pub lat: f32,
    pub lon: f32,
}

impl ToString for Coordinates {
    fn to_string(&self) -> String {
        format!("{},{}", self.lat, self.lon)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinatesAsString {
    pub lat: String,
    pub lon: String,
}

impl TryFrom<CoordinatesAsString> for Coordinates {
    type Error = LocationError;

    fn try_from(value: CoordinatesAsString) -> Result<Self, Self::Error> {
        let lat = value
            .lat
            .parse::<f32>()
            .map_err(|_| LocationError::new(format!("Could not parse latitude: {}", value.lat)))?;
        let lon = value
            .lon
            .parse::<f32>()
            .map_err(|_| LocationError::new(format!("Could not parse longitude: {}", value.lon)))?;
        Ok(Self { lat, lon })
    }
}

impl Coordinates {
    pub fn new(lat: f32, lon: f32) -> Self {
        Self { lat, lon }
    }
}

impl TryFrom<Value> for Coordinates {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}
