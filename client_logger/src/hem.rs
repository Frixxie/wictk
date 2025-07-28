use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use wictk_core::{Lightning, Nowcast};

#[derive(Debug)]
pub struct SensorIds {
    pub temperature: i32,
    pub humidity: i32,
    pub wind_speed: i32,
    pub wind_deg: i32,
    pub lon: i32,
    pub lat: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sensor {
    #[serde(skip_serializing)]
    id: i32,
    name: String,
    unit: String,
}

pub type DeviceId = i32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    #[serde(skip_serializing)]
    id: i32,
    name: String,
    location: String,
}

#[derive(Serialize, Debug)]
pub struct Measurement {
    timestamp: Option<DateTime<Utc>>,
    device: i32,
    sensor: i32,
    measurement: f32,
}

impl Measurement {
    pub fn new(device: i32, sensor: i32, measurement: f32) -> Self {
        Self {
            timestamp: None,
            device,
            sensor,
            measurement,
        }
    }

    pub fn new_with_ts(ts: DateTime<Utc>, device: i32, sensor: i32, measurement: f32) -> Self {
        Self {
            timestamp: Some(ts),
            device,
            sensor,
            measurement,
        }
    }
}

pub fn fetch_devices(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<Device>> {
    let devices = client.get(url).send()?.json::<Vec<Device>>()?;
    Ok(devices)
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
            info!("{:?}", d);
            Ok(d.id)
        }
        None => {
            let new_device = Sensor {
                id: 0,
                name: sensor_name.to_string(),
                unit: sensor_unit.to_string(),
            };
            let response = client.post(url).json(&new_device).send()?;
            info!("{:?}", response);
            setup_sensor(client, url, sensor_name, sensor_unit)
        }
    }
}

pub fn setup_sensors(client: &reqwest::blocking::Client, url: &str) -> Result<SensorIds> {
    let temperature_id = setup_sensor(client, url, "temperature", "°C")?;
    let humidity_id = setup_sensor(client, url, "humidity", "%")?;
    let wind_speed_id = setup_sensor(client, url, "wind_speed", "m/s")?;
    let wind_deg_id = setup_sensor(client, url, "wind_deg", "°")?;
    let lon_id = setup_sensor(client, url, "lon", "°")?;
    let lat_id = setup_sensor(client, url, "lat", "°")?;

    Ok(SensorIds {
        temperature: temperature_id,
        humidity: humidity_id,
        wind_speed: wind_speed_id,
        wind_deg: wind_deg_id,
        lon: lon_id,
        lat: lat_id,
    })
}

pub fn setup_device(
    client: &reqwest::blocking::Client,
    url: &str,
    device_name: &str,
    device_location: &str,
) -> Result<DeviceId> {
    let devices = fetch_devices(client, url)?;
    let device = devices
        .iter()
        .find(|d| d.name == device_name && d.location == device_location);
    match device {
        Some(d) => {
            info!("{:?}", d);
            Ok(d.id)
        }
        None => {
            let new_device = Device {
                id: 0,
                name: device_name.to_string(),
                location: device_location.to_string(),
            };
            let response = client.post(url).json(&new_device).send()?;
            info!("{:?}", response);
            setup_device(client, url, device_name, device_location)
        }
    }
}

pub fn store_nowcast(
    client: &reqwest::blocking::Client,
    url: &str,
    nowcast: &Nowcast,
    device_id: &DeviceId,
    sensor_ids: &SensorIds,
) -> Result<()> {
    match nowcast {
        Nowcast::Met(met_nowcast) => {
            info!("Logging MET");
            let temperature = Measurement::new(
                *device_id,
                sensor_ids.temperature,
                met_nowcast.air_temperature,
            );
            let humidity = Measurement::new(
                *device_id,
                sensor_ids.humidity,
                met_nowcast.relative_humidity,
            );
            let wind_speed =
                Measurement::new(*device_id, sensor_ids.wind_speed, met_nowcast.wind_speed);
            let wind_deg = Measurement::new(
                *device_id,
                sensor_ids.wind_deg,
                met_nowcast.wind_from_direction,
            );
            client
                .post(url)
                .json(&vec![&temperature, &humidity, &wind_speed, &wind_deg])
                .send()?;
        }
        Nowcast::OpenWeather(open_weather_nowcast) => {
            info!("Logging OpenWeather");
            let temperature = Measurement::new(
                *device_id,
                sensor_ids.temperature,
                open_weather_nowcast.temp,
            );
            let humidity = Measurement::new(
                *device_id,
                sensor_ids.humidity,
                open_weather_nowcast.humidity as f32,
            );
            let wind_speed = Measurement::new(
                *device_id,
                sensor_ids.wind_speed,
                open_weather_nowcast.wind_speed,
            );
            let wind_deg = Measurement::new(
                *device_id,
                sensor_ids.wind_deg,
                open_weather_nowcast.wind_deg as f32,
            );
            client
                .post(url)
                .json(&vec![&temperature, &humidity, &wind_speed, &wind_deg])
                .send()?;
        }
    }
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
    let measurement_1 = Measurement::new_with_ts(
        lightning.time,
        *device_id,
        lat_id,
        lightning.location.x() as f32,
    );
    let measurement_2 = Measurement::new_with_ts(
        lightning.time,
        *device_id,
        lon_id,
        lightning.location.y() as f32,
    );
    client
        .post(url)
        .json(&[measurement_1, measurement_2])
        .send()?;
    Ok(())
}
