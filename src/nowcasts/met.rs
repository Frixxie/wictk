use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

use crate::locations::Coordinates;

use super::{Nowcast, NowcastError, NowcastFetcher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetNowcast {
    pub time: DateTime<Utc>,
    pub location: Coordinates,
    pub description: String,
    pub air_temperature: f32,
    pub relative_humidity: f32,
    pub precipitation_rate: f32,
    pub precipitation_amount: f32,
    pub wind_speed: f32,
    pub wind_speed_gust: f32,
    pub wind_from_direction: f32,
}

impl TryFrom<serde_json::Value> for MetNowcast {
    type Error = NowcastError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let location = value["geometry"]["coordinates"]
            .as_array()
            .ok_or_else(|| NowcastError::new("Could not find location"))?;
        let location = Coordinates::new(
            location[0].as_f64().unwrap() as f32,
            location[1].as_f64().unwrap() as f32,
        );

        let time = value["properties"]["meta"]["updated_at"]
            .as_str()
            .ok_or_else(|| NowcastError::new("Could not find time"))?;

        let description = value["properties"]["timeseries"][0]["data"]["next_1_hours"]["summary"]
            ["symbol_code"]
            .as_str()
            .ok_or_else(|| NowcastError::new("Could not find description"))?;

        let air_temperature = value["properties"]["timeseries"][0]["data"]["instant"]["details"]
            ["air_temperature"]
            .as_f64()
            .ok_or_else(|| NowcastError::new("Could not find air_temperature"))?;

        let relative_humidity = value["properties"]["timeseries"][0]["data"]["instant"]["details"]
            ["relative_humidity"]
            .as_f64()
            .ok_or_else(|| NowcastError::new("Could not find relative_humidity"))?;

        let precipitation_amount = value["properties"]["timeseries"][0]["data"]["next_1_hours"]
            ["details"]["precipitation_amount"]
            .as_f64()
            .ok_or_else(|| NowcastError::new("Could not find precipitation_amount"))?;

        let wind_speed = value["properties"]["timeseries"][0]["data"]["instant"]["details"]
            ["wind_speed"]
            .as_f64()
            .ok_or_else(|| NowcastError::new("Could not find wind_speed"))?;

        let wind_speed_gust = value["properties"]["timeseries"][0]["data"]["instant"]["details"]
            ["wind_speed_of_gust"]
            .as_f64()
            .ok_or_else(|| NowcastError::new("Could not find wind_speed_of_gust"))?;

        let wind_from_direction = value["properties"]["timeseries"][0]["data"]["instant"]
            ["details"]["wind_from_direction"]
            .as_f64()
            .ok_or_else(|| NowcastError::new("Could not find wind_from_direction"))?;

        Ok(Self {
            location,
            time: time
                .to_string()
                .parse()
                .map_err(|_| NowcastError::new("Failed to parse time"))?,
            description: description.to_string(),
            air_temperature: air_temperature as f32,
            relative_humidity: relative_humidity as f32,
            precipitation_rate: precipitation_amount as f32,
            precipitation_amount: precipitation_amount as f32,
            wind_speed: wind_speed as f32,
            wind_speed_gust: wind_speed_gust as f32,
            wind_from_direction: wind_from_direction as f32,
        })
    }
}

impl From<MetNowcast> for Nowcast {
    fn from(met: MetNowcast) -> Self {
        Self::Met(met)
    }
}

impl NowcastFetcher for MetNowcast {
    async fn fetch(client: &Client, location: &Coordinates) -> Result<Nowcast, NowcastError> {
        let met_cast: MetNowcast = client
            .get("https://api.met.no/weatherapi/nowcast/2.0/complete")
            .query(&[("lat", location.lat), ("lon", location.lon)])
            .send()
            .await
            .map_err(|err| {
                error!("Error {}", err);
                NowcastError::new("Request to Met.no failed")
            })?
            .json::<Value>()
            .await
            .map_err(|err| {
                error!("Error {}", err);
                NowcastError::new("Deserialization from Met.no failed")
            })?
            .try_into()
            .map_err(|err| {
                error!("Error {}", err);
                NowcastError::new("Failed to convert from met value into nowcast type")
            })?;
        Ok(met_cast.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn met_test_from_value() {
        let json = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[10.4034,63.4308,0]},"properties":{"meta":{"updated_at":"2023-08-14T18:16:07Z","units":{"air_temperature":"celsius","precipitation_amount":"mm","precipitation_rate":"mm/h","relative_humidity":"%","wind_from_direction":"degrees","wind_speed":"m/s","wind_speed_of_gust":"m/s"},"radar_coverage":"ok"},"timeseries":[{"time":"2023-08-14T18:15:00Z","data":{"instant":{"details":{"air_temperature":17.7,"precipitation_rate":0.0,"relative_humidity":80.5,"wind_from_direction":294.4,"wind_speed":2.7,"wind_speed_of_gust":6.1}},"next_1_hours":{"summary":{"symbol_code":"cloudy"},"details":{"precipitation_amount":0.0}}}},{"time":"2023-08-14T18:20:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:25:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:30:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:35:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:40:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:45:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:50:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:55:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T19:00:00Z","data":{"instant":{"details":{"precipitation_rate":0.2}}}},{"time":"2023-08-14T19:05:00Z","data":{"instant":{"details":{"precipitation_rate":0.5}}}},{"time":"2023-08-14T19:10:00Z","data":{"instant":{"details":{"precipitation_rate":0.7}}}},{"time":"2023-08-14T19:15:00Z","data":{"instant":{"details":{"precipitation_rate":0.9}}}},{"time":"2023-08-14T19:20:00Z","data":{"instant":{"details":{"precipitation_rate":1.1}}}},{"time":"2023-08-14T19:25:00Z","data":{"instant":{"details":{"precipitation_rate":1.4}}}},{"time":"2023-08-14T19:30:00Z","data":{"instant":{"details":{"precipitation_rate":1.6}}}},{"time":"2023-08-14T19:35:00Z","data":{"instant":{"details":{"precipitation_rate":1.8}}}},{"time":"2023-08-14T19:40:00Z","data":{"instant":{"details":{"precipitation_rate":1.8}}}},{"time":"2023-08-14T19:45:00Z","data":{"instant":{"details":{"precipitation_rate":1.9}}}},{"time":"2023-08-14T19:50:00Z","data":{"instant":{"details":{"precipitation_rate":1.9}}}},{"time":"2023-08-14T19:55:00Z","data":{"instant":{"details":{"precipitation_rate":1.9}}}},{"time":"2023-08-14T20:00:00Z","data":{"instant":{"details":{"precipitation_rate":2.0}}}},{"time":"2023-08-14T20:05:00Z","data":{"instant":{"details":{"precipitation_rate":2.5}}}}]}}"#;

        let json_value: serde_json::Value = serde_json::from_str(json).unwrap();

        let met = MetNowcast::try_from(json_value).unwrap();

        assert_eq!(met.location.lat, 63.4308);
        assert_eq!(met.location.lon, 10.4034);
        assert_eq!(met.time.to_string(), "2023-08-14 18:16:07 UTC");
        assert_eq!(met.description, "cloudy");
        assert_eq!(met.air_temperature, 17.7);
        assert_eq!(met.relative_humidity, 80.5);
        assert_eq!(met.precipitation_amount, 0.0);
        assert_eq!(met.wind_speed, 2.7);
        assert_eq!(met.wind_speed_gust, 6.1);
        assert_eq!(met.wind_from_direction, 294.4);
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
        assert!(nowcast.is_ok())
    }
}
