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

#[cfg(test)]

mod tests {
    use crate::{
        locations::Coordinates,
        nowcasts::{simple::SimpleNowcast, MetNowcast, NowcastFetcher, OpenWeatherNowcast},
    };

    #[tokio::test]
    async fn openweathermap_fetch_and_decode_into_simple() {
        let client_builder = reqwest::Client::builder();
        static APP_USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
        let location = Coordinates::new(10.4034, 63.4308);
        let nowcast = OpenWeatherNowcast::fetch(&client, &location).await;
        assert!(nowcast.is_ok());

        let nowcast = match nowcast.unwrap() {
            crate::nowcasts::Nowcast::Met(_) => panic!(),
            crate::nowcasts::Nowcast::OpenWeather(res) => res,
            crate::nowcasts::Nowcast::Simple(_) => panic!(),
        };

        let _simple_nowcast: SimpleNowcast = nowcast.into();
    }

    #[tokio::test]
    async fn met_fetch() {
        let client_builder = reqwest::Client::builder();
        static APP_USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
        let location = Coordinates::new(10.4034, 63.4308);
        let nowcast = MetNowcast::fetch(&client, &location).await;
        assert!(nowcast.is_ok());

        let nowcast = match nowcast.unwrap() {
            crate::nowcasts::Nowcast::Met(r) => r,
            crate::nowcasts::Nowcast::OpenWeather(_) => panic!(),
            crate::nowcasts::Nowcast::Simple(_) => panic!(),
        };

        let _simple_nowcast: SimpleNowcast = nowcast.into();
    }
}
