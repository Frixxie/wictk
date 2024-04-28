mod city;
mod coordinates;
mod location;

pub use city::City;
pub use coordinates::Coordinates;
pub use coordinates::CoordinatesAsString;
pub use location::OpenWeatherMapLocation;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::locations::{
        city::City, coordinates::Coordinates, location::OpenWeatherMapLocation,
    };

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

    #[tokio::test]
    async fn test_fetch_location() {
        let client = reqwest::Client::new();
        let res = OpenWeatherMapLocation::fetch(&client, "Oslo").await;
        assert!(res.is_some());
        assert_eq!(res.unwrap().len(), 1);
    }
}
