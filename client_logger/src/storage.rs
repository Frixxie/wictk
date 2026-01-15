use crate::device::DeviceId;
use crate::measurement::NewMeasurement;
use crate::sensor::SensorIds;
use anyhow::Result;
use wictk_core::{Lightning, MetNowcast, OpenWeatherNowcast};

pub fn store_met_nowcast(
    client: &reqwest::blocking::Client,
    url: &str,
    met_nowcast: &MetNowcast,
    device_id: &DeviceId,
    sensor_ids: &SensorIds,
) -> Result<()> {
    tracing::info!("Storing MET nowcast with timestamp: {}", met_nowcast.time);
    let temperature = NewMeasurement::new_with_ts(
        met_nowcast.time,
        *device_id,
        sensor_ids.temperature,
        met_nowcast.air_temperature,
    );
    let humidity = NewMeasurement::new_with_ts(
        met_nowcast.time,
        *device_id,
        sensor_ids.humidity,
        met_nowcast.relative_humidity,
    );
    let wind_speed = NewMeasurement::new_with_ts(
        met_nowcast.time,
        *device_id,
        sensor_ids.wind_speed,
        met_nowcast.wind_speed,
    );
    let wind_deg = NewMeasurement::new_with_ts(
        met_nowcast.time,
        *device_id,
        sensor_ids.wind_deg,
        met_nowcast.wind_from_direction,
    );
    let precipitation_rate = NewMeasurement::new_with_ts(
        met_nowcast.time,
        *device_id,
        sensor_ids.precipitation_rate,
        met_nowcast.precipitation_rate,
    );
    let precipitation_amount = NewMeasurement::new_with_ts(
        met_nowcast.time,
        *device_id,
        sensor_ids.precipitation_amount,
        met_nowcast.precipitation_amount,
    );
    let wind_speed_gust = NewMeasurement::new_with_ts(
        met_nowcast.time,
        *device_id,
        sensor_ids.wind_speed_gust,
        met_nowcast.wind_speed_gust,
    );
    client
        .post(url)
        .json(&vec![
            &temperature,
            &humidity,
            &wind_speed,
            &wind_deg,
            &precipitation_rate,
            &precipitation_amount,
            &wind_speed_gust,
        ])
        .send()?
        .error_for_status()?;
    Ok(())
}

pub fn store_openweather_nowcast(
    client: &reqwest::blocking::Client,
    url: &str,
    open_weather_nowcast: &OpenWeatherNowcast,
    device_id: &DeviceId,
    sensor_ids: &SensorIds,
) -> Result<()> {
    tracing::info!(
        "Storing OpenWeather nowcast with timestamp: {}",
        open_weather_nowcast.dt
    );
    let temperature = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.temperature,
        open_weather_nowcast.temp,
    );
    let humidity = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.humidity,
        open_weather_nowcast.humidity as f32,
    );
    let wind_speed = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.wind_speed,
        open_weather_nowcast.wind_speed,
    );
    let wind_deg = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.wind_deg,
        open_weather_nowcast.wind_deg as f32,
    );
    let feels_like = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.feels_like,
        open_weather_nowcast.feels_like,
    );
    let pressure = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.pressure,
        open_weather_nowcast.pressure as f32,
    );
    let clouds = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.clouds,
        open_weather_nowcast.clouds as f32,
    );
    let visibility = NewMeasurement::new_with_ts(
        open_weather_nowcast.dt,
        *device_id,
        sensor_ids.visibility,
        open_weather_nowcast.visibility as f32,
    );
    client
        .post(url)
        .json(&vec![
            &temperature,
            &humidity,
            &wind_speed,
            &wind_deg,
            &feels_like,
            &pressure,
            &clouds,
            &visibility,
        ])
        .send()?
        .error_for_status()?;
    Ok(())
}

pub fn store_lightning(
    client: &reqwest::blocking::Client,
    url: &str,
    device_id: &DeviceId,
    lon_id: i32,
    lat_id: i32,
    lightning: &Lightning,
) -> Result<()> {
    let measurement_1 = NewMeasurement::new_with_ts(
        lightning.time,
        *device_id,
        lat_id,
        lightning.location.x() as f32,
    );
    let measurement_2 = NewMeasurement::new_with_ts(
        lightning.time,
        *device_id,
        lon_id,
        lightning.location.y() as f32,
    );
    client
        .post(url)
        .json(&[measurement_1, measurement_2])
        .send()?
        .error_for_status()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use geo::Point;
    use mockito::Server;
    use wictk_core::{MetNowcast, OpenWeatherNowcast};

    #[test]
    fn should_store_met_nowcast() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/")
            .with_status(200)
            .match_body(mockito::Matcher::JsonString(
                r#"[{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":1,"measurement":20.5},{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":2,"measurement":65.0},{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":3,"measurement":5.2},{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":4,"measurement":180.0},{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":5,"measurement":0.0},{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":6,"measurement":0.0},{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":7,"measurement":6.0}]"#.to_string()
            ))
            .create();

        let client = reqwest::blocking::Client::new();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let met_nowcast = MetNowcast {
            time: timestamp,
            location: wictk_core::Coordinates {
                lon: 10.0,
                lat: 63.0,
            },
            description: "Clear".to_string(),
            air_temperature: 20.5,
            relative_humidity: 65.0,
            precipitation_rate: 0.0,
            precipitation_amount: 0.0,
            wind_speed: 5.2,
            wind_speed_gust: 6.0,
            wind_from_direction: 180.0,
        };

        let device_id = 1;
        let sensor_ids = SensorIds {
            temperature: 1,
            humidity: 2,
            wind_speed: 3,
            wind_deg: 4,
            precipitation_rate: 5,
            precipitation_amount: 6,
            wind_speed_gust: 7,
            feels_like: 8,
            pressure: 9,
            clouds: 10,
            visibility: 11,
            lon: 12,
            lat: 13,
        };

        let result = store_met_nowcast(
            &client,
            &server.url(),
            &met_nowcast,
            &device_id,
            &sensor_ids,
        );
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn should_store_openweather_nowcast() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/").with_status(200).create();

        let client = reqwest::blocking::Client::new();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let openweather_nowcast = OpenWeatherNowcast {
            dt: timestamp,
            name: "Trondheim".to_string(),
            country: "NO".to_string(),
            lon: 10.0,
            lat: 63.0,
            main: "Clear".to_string(),
            desc: "clear sky".to_string(),
            clouds: 0,
            wind_speed: 4.1,
            wind_deg: 200,
            visibility: 10000,
            temp: 22.3,
            feels_like: 23.0,
            humidity: 70,
            pressure: 1013,
        };

        let device_id = 1;
        let sensor_ids = SensorIds {
            temperature: 1,
            humidity: 2,
            wind_speed: 3,
            wind_deg: 4,
            precipitation_rate: 5,
            precipitation_amount: 6,
            wind_speed_gust: 7,
            feels_like: 8,
            pressure: 9,
            clouds: 10,
            visibility: 11,
            lon: 12,
            lat: 13,
        };

        let result = store_openweather_nowcast(
            &client,
            &server.url(),
            &openweather_nowcast,
            &device_id,
            &sensor_ids,
        );
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn should_store_lightning() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/")
            .with_status(200)
            .match_body(mockito::Matcher::JsonString(
                r#"[{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":13,"measurement":10.0},{"timestamp":"2025-08-11T12:00:00Z","device":1,"sensor":12,"measurement":63.0}]"#.to_string()
            ))
            .create();

        let client = reqwest::blocking::Client::new();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let lightning = Lightning {
            time: timestamp,
            location: Point::new(10.0, 63.0),
            magic_value: 42,
        };

        let device_id = 1;
        let lon_id = 12;
        let lat_id = 13;

        let result = store_lightning(
            &client,
            &server.url(),
            &device_id,
            lon_id,
            lat_id,
            &lightning,
        );
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn should_handle_store_met_nowcast_error() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/").with_status(500).create();

        let client = reqwest::blocking::Client::new();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let met_nowcast = MetNowcast {
            time: timestamp,
            location: wictk_core::Coordinates {
                lon: 10.0,
                lat: 63.0,
            },
            description: "Clear".to_string(),
            air_temperature: 20.5,
            relative_humidity: 65.0,
            precipitation_rate: 0.0,
            precipitation_amount: 0.0,
            wind_speed: 5.2,
            wind_speed_gust: 6.0,
            wind_from_direction: 180.0,
        };

        let device_id = 1;
        let sensor_ids = SensorIds {
            temperature: 1,
            humidity: 2,
            wind_speed: 3,
            wind_deg: 4,
            precipitation_rate: 5,
            precipitation_amount: 6,
            wind_speed_gust: 7,
            feels_like: 8,
            pressure: 9,
            clouds: 10,
            visibility: 11,
            lon: 12,
            lat: 13,
        };

        let result = store_met_nowcast(
            &client,
            &server.url(),
            &met_nowcast,
            &device_id,
            &sensor_ids,
        );
        assert!(
            result.is_err(),
            "Expected error for HTTP 500 but got: {result:?}"
        );
        mock.assert();
    }

    #[test]
    fn should_handle_store_openweather_nowcast_error() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/").with_status(500).create();

        let client = reqwest::blocking::Client::new();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let openweather_nowcast = OpenWeatherNowcast {
            dt: timestamp,
            name: "Trondheim".to_string(),
            country: "NO".to_string(),
            lon: 10.0,
            lat: 63.0,
            main: "Clear".to_string(),
            desc: "clear sky".to_string(),
            clouds: 0,
            wind_speed: 4.1,
            wind_deg: 200,
            visibility: 10000,
            temp: 22.3,
            feels_like: 23.0,
            humidity: 70,
            pressure: 1013,
        };

        let device_id = 1;
        let sensor_ids = SensorIds {
            temperature: 1,
            humidity: 2,
            wind_speed: 3,
            wind_deg: 4,
            precipitation_rate: 5,
            precipitation_amount: 6,
            wind_speed_gust: 7,
            feels_like: 8,
            pressure: 9,
            clouds: 10,
            visibility: 11,
            lon: 12,
            lat: 13,
        };

        let result = store_openweather_nowcast(
            &client,
            &server.url(),
            &openweather_nowcast,
            &device_id,
            &sensor_ids,
        );
        assert!(
            result.is_err(),
            "Expected error for HTTP 500 but got: {result:?}"
        );
        mock.assert();
    }

    #[test]
    fn should_handle_store_lightning_error() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/").with_status(500).create();

        let client = reqwest::blocking::Client::new();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2025-08-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let lightning = Lightning {
            time: timestamp,
            location: Point::new(10.0, 63.0),
            magic_value: 42,
        };

        let device_id = 1;
        let lon_id = 12;
        let lat_id = 13;

        let result = store_lightning(
            &client,
            &server.url(),
            &device_id,
            lon_id,
            lat_id,
            &lightning,
        );
        assert!(
            result.is_err(),
            "Expected error for HTTP 500 but got: {result:?}"
        );
        mock.assert();
    }
}
