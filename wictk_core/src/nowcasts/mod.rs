mod met;
mod openweathermap;

pub use met::MetNowcast;
pub use openweathermap::OpenWeatherNowcast;

use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Nowcast {
    Met(MetNowcast),
    OpenWeather(OpenWeatherNowcast),
}

impl Display for Nowcast {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Nowcast::Met(met_nowcast) => write!(
                f,
                "{}°C, {}%, {} m/s, {}°",
                met_nowcast.air_temperature,
                met_nowcast.relative_humidity,
                met_nowcast.wind_speed,
                met_nowcast.wind_from_direction,
            ),
            Nowcast::OpenWeather(open_weather_nowcast) => write!(
                f,
                "{}°C, {}%, {} m/s, {}°",
                open_weather_nowcast.temp,
                open_weather_nowcast.humidity,
                open_weather_nowcast.wind_speed,
                open_weather_nowcast.wind_deg,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowcastError {
    pub message: String,
}

impl NowcastError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for NowcastError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NowcastError: {}", self.message)
    }
}

impl Error for NowcastError {}
