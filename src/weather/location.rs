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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LocationQuery {
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LocationType {
    Location(LocationQuery),
    Coordinates(Location),
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_location_query() {
        let location_query = LocationQuery {
            location: "Oslo".to_string(),
        };
        assert_eq!(location_query.location, "Oslo".to_string());
    }

    #[test]
    fn test_locationtype_location() {
        let json = r#"{"lat": 1.0, "lon": 2.0}"#;
        let location: LocationType = serde_json::from_str(json).unwrap();
        assert_eq!(location, LocationType::Coordinates(Location::new(1.0, 2.0)));
    }

    #[test]
    fn test_locationtype_query() {
        let json = r#"{"location": "Oslo"}"#;
        let location: LocationType = serde_json::from_str(json).unwrap();
        assert_eq!(
            location,
            LocationType::Location(LocationQuery {
                location: "Oslo".to_string()
            })
        );
    }
}
