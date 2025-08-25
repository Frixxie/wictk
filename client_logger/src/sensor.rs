use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct SensorIds {
    pub temperature: i32,
    pub humidity: i32,
    pub wind_speed: i32,
    pub wind_deg: i32,
    pub precipitation_rate: i32,
    pub precipitation_amount: i32,
    pub wind_speed_gust: i32,
    pub feels_like: i32,
    pub pressure: i32,
    pub clouds: i32,
    pub visibility: i32,
    pub lon: i32,
    pub lat: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Sensor {
    #[serde(skip_serializing)]
    pub id: i32,
    pub name: String,
    pub unit: String,
}

pub fn fetch_sensors(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<Sensor>> {
    let devices = client.get(url).send()?.json::<Vec<Sensor>>()?;
    Ok(devices)
}

fn setup_sensor(
    client: &reqwest::blocking::Client,
    url: &str,
    sensor_name: &str,
    sensor_unit: &str,
) -> Result<i32> {
    let sensors = fetch_sensors(client, url)?;
    let device = sensors.iter().find(|d| d.name == sensor_name);
    match device {
        Some(d) => {
            tracing::info!("Found existing sensor: {:?}", d);
            Ok(d.id)
        }
        None => {
            let new_device = Sensor {
                id: 0,
                name: sensor_name.to_string(),
                unit: sensor_unit.to_string(),
            };
            let response = client.post(url).json(&new_device).send()?;
            tracing::info!("Created new sensor: {:?}", response);
            setup_sensor(client, url, sensor_name, sensor_unit)
        }
    }
}

pub fn setup_sensors(client: &reqwest::blocking::Client, url: &str) -> Result<SensorIds> {
    let temperature_id = setup_sensor(client, url, "temperature", "°C")?;
    let humidity_id = setup_sensor(client, url, "humidity", "%")?;
    let wind_speed_id = setup_sensor(client, url, "wind_speed", "m/s")?;
    let wind_deg_id = setup_sensor(client, url, "wind_deg", "°")?;
    let precipitation_rate_id = setup_sensor(client, url, "precipitation_rate", "mm/h")?;
    let precipitation_amount_id = setup_sensor(client, url, "precipitation_amount", "mm")?;
    let wind_speed_gust_id = setup_sensor(client, url, "wind_speed_gust", "m/s")?;
    let feels_like_id = setup_sensor(client, url, "feels_like", "°C")?;
    let pressure_id = setup_sensor(client, url, "pressure", "hPa")?;
    let clouds_id = setup_sensor(client, url, "clouds", "%")?;
    let visibility_id = setup_sensor(client, url, "visibility", "m")?;
    let lon_id = setup_sensor(client, url, "lon", "°")?;
    let lat_id = setup_sensor(client, url, "lat", "°")?;

    Ok(SensorIds {
        temperature: temperature_id,
        humidity: humidity_id,
        wind_speed: wind_speed_id,
        wind_deg: wind_deg_id,
        precipitation_rate: precipitation_rate_id,
        precipitation_amount: precipitation_amount_id,
        wind_speed_gust: wind_speed_gust_id,
        feels_like: feels_like_id,
        pressure: pressure_id,
        clouds: clouds_id,
        visibility: visibility_id,
        lon: lon_id,
        lat: lat_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn should_fetch_sensors_successfully() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {"id": 1, "name": "temperature", "unit": "°C"},
                {"id": 2, "name": "humidity", "unit": "%"}
            ]"#)
            .create();

        let client = reqwest::blocking::Client::new();
        let result = fetch_sensors(&client, &server.url());
        
        assert!(result.is_ok());
        let sensors = result.unwrap();
        assert_eq!(sensors.len(), 2);
        assert_eq!(sensors[0].id, 1);
        assert_eq!(sensors[0].name, "temperature");
        assert_eq!(sensors[0].unit, "°C");
        
        mock.assert();
    }

    #[test]
    fn should_compare_sensor_ids_for_equality() {
        let sensor_ids1 = SensorIds {
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
        
        let sensor_ids2 = SensorIds {
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
        
        assert_eq!(sensor_ids1, sensor_ids2);
    }

    #[test]
    fn should_compare_sensors_for_equality() {
        let sensor1 = Sensor {
            id: 1,
            name: "temperature".to_string(),
            unit: "°C".to_string(),
        };
        
        let sensor2 = Sensor {
            id: 1,
            name: "temperature".to_string(),
            unit: "°C".to_string(),
        };
        
        assert_eq!(sensor1, sensor2);
    }

    #[test]
    fn should_serialize_sensor_correctly() {
        let sensor = Sensor {
            id: 1,
            name: "temperature".to_string(),
            unit: "°C".to_string(),
        };
        
        let json = serde_json::to_string(&sensor).unwrap();
        let expected = r#"{"name":"temperature","unit":"°C"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn should_setup_sensors_successfully() {
        let mut server = Server::new();
        
        let mock_get = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {"id": 1, "name": "temperature", "unit": "°C"},
                {"id": 2, "name": "humidity", "unit": "%"},
                {"id": 3, "name": "wind_speed", "unit": "m/s"},
                {"id": 4, "name": "wind_deg", "unit": "°"},
                {"id": 5, "name": "precipitation_rate", "unit": "mm/h"},
                {"id": 6, "name": "precipitation_amount", "unit": "mm"},
                {"id": 7, "name": "wind_speed_gust", "unit": "m/s"},
                {"id": 8, "name": "feels_like", "unit": "°C"},
                {"id": 9, "name": "pressure", "unit": "hPa"},
                {"id": 10, "name": "clouds", "unit": "%"},
                {"id": 11, "name": "visibility", "unit": "m"},
                {"id": 12, "name": "lon", "unit": "°"},
                {"id": 13, "name": "lat", "unit": "°"}
            ]"#)
            .expect(13)
            .create();

        let client = reqwest::blocking::Client::new();
        let result = setup_sensors(&client, &server.url());
        
        assert!(result.is_ok());
        let sensor_ids = result.unwrap();
        assert_eq!(sensor_ids.temperature, 1);
        assert_eq!(sensor_ids.humidity, 2);
        assert_eq!(sensor_ids.wind_speed, 3);
        assert_eq!(sensor_ids.wind_deg, 4);
        assert_eq!(sensor_ids.precipitation_rate, 5);
        assert_eq!(sensor_ids.precipitation_amount, 6);
        assert_eq!(sensor_ids.wind_speed_gust, 7);
        assert_eq!(sensor_ids.feels_like, 8);
        assert_eq!(sensor_ids.pressure, 9);
        assert_eq!(sensor_ids.clouds, 10);
        assert_eq!(sensor_ids.visibility, 11);
        assert_eq!(sensor_ids.lon, 12);
        assert_eq!(sensor_ids.lat, 13);
        
        mock_get.assert();
    }
}