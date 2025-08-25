use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Measurement {
    pub timestamp: Option<DateTime<Utc>>,
    pub device: i32,
    pub sensor: i32,
    pub measurement: f32,
}

impl Measurement {
    pub fn new_with_ts(ts: DateTime<Utc>, device: i32, sensor: i32, measurement: f32) -> Self {
        Self {
            timestamp: Some(ts),
            device,
            sensor,
            measurement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn should_create_measurement_with_timestamp() {
        let timestamp = Utc::now();
        let measurement = Measurement::new_with_ts(timestamp, 1, 2, 25.5);
        
        assert_eq!(measurement.timestamp, Some(timestamp));
        assert_eq!(measurement.device, 1);
        assert_eq!(measurement.sensor, 2);
        assert_eq!(measurement.measurement, 25.5);
    }

    #[test]
    fn should_compare_measurements_for_equality() {
        let timestamp = Utc::now();
        let measurement1 = Measurement::new_with_ts(timestamp, 1, 2, 25.5);
        let measurement2 = Measurement::new_with_ts(timestamp, 1, 2, 25.5);
        
        assert_eq!(measurement1, measurement2);
    }

    #[test]
    fn should_serialize_measurement_correctly() {
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let measurement = Measurement::new_with_ts(timestamp, 1, 2, 25.5);
        
        let json = serde_json::to_string(&measurement).unwrap();
        let expected = r#"{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":2,"measurement":25.5}"#;
        assert_eq!(json, expected);
    }
}