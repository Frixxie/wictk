use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
pub struct Location {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationString {
    pub lat: String,
    pub lon: String,
}

impl TryFrom<LocationString> for Location {
    type Error = LocationError;

    fn try_from(value: LocationString) -> Result<Self, Self::Error> {
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

impl Location {
    pub fn new(lat: f32, lon: f32) -> Self {
        Self { lat, lon }
    }
}

impl TryFrom<Value> for Location {
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
pub enum LocationType {
    Location(City),
    Coordinates(Location),
    CoordinatesString(LocationString),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_location() {
        let location = Location::new(1.0, 2.0);
        assert_eq!(location.lat, 1.0);
        assert_eq!(location.lon, 2.0);
    }

    #[test]
    fn deserialize_location() {
        let json = r#"{"lat": 1.0, "lon": 2.0}"#;
        let location: Location = serde_json::from_str(json).unwrap();
        assert_eq!(location.lat, 1.0);
        assert_eq!(location.lon, 2.0);
    }

    #[test]
    fn serialize_location() {
        let location = Location::new(1.0, 2.0);
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
    fn test_locationtype_coordinates_floats() {
        let json = r#"{"lat": 1.0, "lon": 2.0}"#;
        let location: LocationType = serde_json::from_str(json).unwrap();
        assert_eq!(location, LocationType::Coordinates(Location::new(1.0, 2.0)));
    }

    #[test]
    fn test_locationtype_coordinates_strings() {
        let json = r#"{"lat": "1.0", "lon": "2.0"}"#;
        let location: LocationType = serde_json::from_str(json).unwrap();
        assert_eq!(
            location,
            LocationType::CoordinatesString(LocationString {
                lat: "1.0".to_string(),
                lon: "2.0".to_string()
            })
        );
    }

    #[test]
    fn test_locationtype_city() {
        let json = r#"{"location": "Oslo"}"#;
        let location: LocationType = serde_json::from_str(json).unwrap();
        assert_eq!(
            location,
            LocationType::Location(City {
                location: "Oslo".to_string()
            })
        );
    }
}
