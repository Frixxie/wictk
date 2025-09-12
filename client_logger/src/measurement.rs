use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct NewMeasurement {
    pub timestamp: Option<DateTime<Utc>>,
    pub device: i32,
    pub sensor: i32,
    pub measurement: f32,
}

impl NewMeasurement {
    pub fn new_with_ts(ts: DateTime<Utc>, device: i32, sensor: i32, measurement: f32) -> Self {
        Self {
            timestamp: Some(ts),
            device,
            sensor,
            measurement,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Measurement {
    pub timestamp: DateTime<Utc>,
    pub value: f32,
    pub unit: String,
    pub device_name: String,
    pub device_location: String,
    pub sensor_name: String,
}

pub fn fetch_measurements(
    client: &reqwest::blocking::Client,
    url: &str,
    device_id: i32,
    sensor_id: i32,
) -> Result<Measurement> {
    let full_url = format!("{url}api/devices/{device_id}/sensors/{sensor_id}/measurements/latest");
    tracing::info!("Fetching measurements from: {}", full_url);

    let response = client.get(&full_url).send()?;

    if response.status().is_success() {
        let measurement: Measurement = response.json()?;
        tracing::info!(
            "Successfully fetched measurement for device {} sensor {}: value={} {} at {} (device: {}, location: {}, sensor: {})",
            device_id,
            sensor_id,
            measurement.value,
            measurement.unit,
            measurement.timestamp,
            measurement.device_name,
            measurement.device_location,
            measurement.sensor_name
        );
        Ok(measurement)
    } else {
        tracing::error!("Failed to fetch measurements: HTTP {}", response.status());
        Err(anyhow::anyhow!("HTTP error: {}", response.status()))
    }
}

pub fn calculate_temperature_ratio(outside_celsius: f32, inside_celsius: f32) -> f32 {
    const KELVIN: f32 = 273.15;
    let outside_kelvin = outside_celsius + KELVIN;
    let inside_kelvin = inside_celsius + KELVIN;
    inside_kelvin / outside_kelvin
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn should_create_measurement_with_timestamp() {
        let timestamp = Utc::now();
        let measurement = NewMeasurement::new_with_ts(timestamp, 1, 2, 25.5);

        assert_eq!(measurement.timestamp, Some(timestamp));
        assert_eq!(measurement.device, 1);
        assert_eq!(measurement.sensor, 2);
        assert_eq!(measurement.measurement, 25.5);
    }

    #[test]
    fn should_compare_measurements_for_equality() {
        let timestamp = Utc::now();
        let measurement1 = NewMeasurement::new_with_ts(timestamp, 1, 2, 25.5);
        let measurement2 = NewMeasurement::new_with_ts(timestamp, 1, 2, 25.5);

        assert_eq!(measurement1, measurement2);
    }

    #[test]
    fn should_serialize_measurement_correctly() {
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let measurement = NewMeasurement::new_with_ts(timestamp, 1, 2, 25.5);

        let json = serde_json::to_string(&measurement).unwrap();
        let expected =
            r#"{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":2,"measurement":25.5}"#;
        assert_eq!(json, expected);
    }
}
