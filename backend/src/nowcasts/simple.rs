use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::locations::Coordinates;

use super::{MetNowcast, OpenWeatherNowcast};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleNowcast {
    time: DateTime<Utc>,
    location: Coordinates,
    temperature: f32,
    humidity: f32,
    wind_speed: f32,
    wind_dir: f32,
}

impl From<MetNowcast> for SimpleNowcast {
    fn from(value: MetNowcast) -> Self {
        Self {
            time: value.time,
            location: value.location,
            temperature: value.air_temperature,
            humidity: value.relative_humidity,
            wind_speed: value.wind_speed,
            wind_dir: value.wind_from_direction,
        }
    }
}

impl From<OpenWeatherNowcast> for SimpleNowcast {
    fn from(value: OpenWeatherNowcast) -> Self {
        Self {
            time: value.dt,
            location: Coordinates::new(value.lat, value.lon),
            temperature: value.temp,
            humidity: value.humidity as f32,
            wind_speed: value.wind_speed,
            wind_dir: value.wind_deg as f32,
        }
    }
}
