use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug)]
pub struct SensorIds {
    pub temperature: i32,
    pub humidity: i32,
    pub wind_speed: i32,
    pub wind_deg: i32,
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

    Ok(SensorIds {
        temperature: temperature_id,
        humidity: humidity_id,
        wind_speed: wind_speed_id,
        wind_deg: wind_deg_id,
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
