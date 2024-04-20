use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub async fn fetch(client: Client, location: String) -> Option<Vec<Self>> {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coordinates {
    pub lat: f32,
    pub lon: f32,
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct City {
    pub location: String,
}

impl TryFrom<Value> for City {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LocationQuery {
    Location(City),
    Coordinates(CoordinatesAsString),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_location() {
        let location = Coordinates::new(1.0, 2.0);
        assert_eq!(location.lat, 1.0);
        assert_eq!(location.lon, 2.0);
    }

    #[test]
    fn deserialize_location() {
        let json = r#"{"lat": 1.0, "lon": 2.0}"#;
        let location: Coordinates = serde_json::from_str(json).unwrap();
        assert_eq!(location.lat, 1.0);
        assert_eq!(location.lon, 2.0);
    }

    #[test]
    fn serialize_location() {
        let location = Coordinates::new(1.0, 2.0);
        let json = serde_json::to_string(&location).unwrap();
        assert_eq!(json, r#"{"lat":1.0,"lon":2.0}"#);
    }

    #[test]
    fn test_location_city() {
        let location_query = City {
            location: "Oslo".to_string(),
        };
        assert_eq!(location_query.location, "Oslo".to_string());
    }

    #[test]
    fn test_locationtype_coordinates_strings() {
        let json = r#"{"lat": "1.0", "lon": "2.0"}"#;
        let location: LocationQuery = serde_json::from_str(json).unwrap();
        assert_eq!(
            location,
            LocationQuery::Coordinates(CoordinatesAsString {
                lat: "1.0".to_string(),
                lon: "2.0".to_string()
            })
        );
    }

    #[test]
    fn test_locationtype_city() {
        let json = r#"{"location": "Oslo"}"#;
        let location: LocationQuery = serde_json::from_str(json).unwrap();
        assert_eq!(
            location,
            LocationQuery::Location(City {
                location: "Oslo".to_string()
            })
        );
    }

    #[tokio::test]
    async fn test_fetch_location() {
        let client = reqwest::Client::new();
        let res = OpenWeatherMapLocation::fetch(client, "Oslo".to_string()).await;
        assert!(res.is_some());
        assert_eq!(res.unwrap().len(), 1);
    }
}
