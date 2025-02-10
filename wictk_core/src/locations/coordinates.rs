use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::location::LocationError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coordinates {
    pub lon: f32,
    pub lat: f32,
}

impl ToString for Coordinates {
    fn to_string(&self) -> String {
        format!("{},{}", self.lon, self.lat)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinatesAsString {
    pub lon: String,
    pub lat: String,
}

impl TryFrom<CoordinatesAsString> for Coordinates {
    type Error = LocationError;

    fn try_from(value: CoordinatesAsString) -> Result<Self, Self::Error> {
        let lon = value
            .lon
            .parse::<f32>()
            .map_err(|_| LocationError::new(format!("Could not parse longitude: {}", value.lon)))?;
        let lat = value
            .lat
            .parse::<f32>()
            .map_err(|_| LocationError::new(format!("Could not parse latitude: {}", value.lat)))?;
        Ok(Self { lon, lat })
    }
}

impl Coordinates {
    pub fn new(lon: f32, lat: f32) -> Self {
        Self { lon, lat }
    }
}

impl TryFrom<Value> for Coordinates {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}
