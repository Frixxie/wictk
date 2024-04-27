use chrono::{DateTime, Utc};
use log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::handlers::location::Coordinates;

use super::{Nowcast, NowcastError, NowcastFetcher};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenWeatherNowcast {
    pub dt: DateTime<Utc>,
    pub name: String,
    pub country: String,
    pub lon: f32,
    pub lat: f32,
    pub main: String,
    pub desc: String,
    pub clouds: u32,
    pub wind_speed: f32,
    pub wind_deg: i32,
    pub visibility: i32,
    pub temp: f32,
    pub feels_like: f32,
    pub humidity: u32,
    pub pressure: u32,
}

impl TryFrom<Value> for OpenWeatherNowcast {
    type Error = NowcastError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let dt = DateTime::from_timestamp(
            value["dt"]
                .as_i64()
                .ok_or(NowcastError::new("Could not find dt"))?,
            0,
        )
        .ok_or(NowcastError::new("Could not find dt"))?;
        let name = value["name"]
            .as_str()
            .ok_or(NowcastError::new("Could not find name"))?
            .to_string();
        let country = value["sys"]["country"]
            .as_str()
            .ok_or(NowcastError::new("Could not find country"))?
            .to_string();
        let lon = value["coord"]["lon"]
            .as_f64()
            .ok_or(NowcastError::new("Could not find lon"))? as f32;
        let lat = value["coord"]["lat"]
            .as_f64()
            .ok_or(NowcastError::new("Could not find lat"))? as f32;
        let main = value["weather"][0]["main"]
            .as_str()
            .ok_or(NowcastError::new("Could not find main"))?
            .to_string();
        let desc = value["weather"][0]["description"]
            .as_str()
            .ok_or(NowcastError::new("Could not find desc"))?
            .to_string();
        let clouds = value["clouds"]["all"]
            .as_u64()
            .ok_or(NowcastError::new("Could not find clouds"))? as u32;
        let wind_speed = value["wind"]["speed"]
            .as_f64()
            .ok_or(NowcastError::new("Could not find wind_speed"))? as f32;
        let wind_deg = value["wind"]["deg"]
            .as_i64()
            .ok_or(NowcastError::new("Could not find wind_deg"))? as i32;
        let visibility = value["visibility"]
            .as_i64()
            .ok_or(NowcastError::new("Could not find visibility"))? as i32;
        let temp = value["main"]["temp"]
            .as_f64()
            .ok_or(NowcastError::new("Could not find temp"))? as f32;
        let feels_like = value["main"]["feels_like"]
            .as_f64()
            .ok_or(NowcastError::new("Could not find feels_like"))? as f32;
        let humidity = value["main"]["humidity"]
            .as_u64()
            .ok_or(NowcastError::new("Could not find humidity"))? as u32;
        let pressure = value["main"]["pressure"]
            .as_u64()
            .ok_or(NowcastError::new("Could not find pressure"))? as u32;

        Ok(Self {
            dt,
            name,
            country,
            lon,
            lat,
            main,
            desc,
            clouds,
            wind_speed,
            wind_deg,
            visibility,
            temp,
            feels_like,
            humidity,
            pressure,
        })
    }
}

impl From<OpenWeatherNowcast> for Nowcast {
    fn from(open_weather: OpenWeatherNowcast) -> Self {
        Self::OpenWeather(open_weather)
    }
}

impl NowcastFetcher for OpenWeatherNowcast {
    async fn fetch(client: &Client, location: &Coordinates) -> Result<Nowcast, NowcastError> {
        let openweathermap: OpenWeatherNowcast = client
            .get("https://api.openweathermap.org/data/2.5/weather")
            .query(&[("lat", location.lat), ("lon", location.lon)])
            .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
            .query(&[("units", "metric")])
            .send()
            .await
            .map_err(|err| {
                error!("Error {}", err);
                NowcastError::new("Request to OpenWeatherMap failed")
            })?
            .json::<Value>()
            .await
            .map_err(|err| {
                error!("Error {}", err);
                NowcastError::new("Deserialization from OpenWeatherMap failed")
            })?
            .try_into()
            .map_err(|err| {
                error!("Error {}", err);
                NowcastError::new("Failed to convert OpenWeatherMap value into nowcast type")
            })?;
        Ok(openweathermap.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        handlers::location::Coordinates,
        nowcasts::{NowcastFetcher, OpenWeatherNowcast},
    };

    #[test]
    fn open_weathermap_from_value() {
        let json = r#"{"coord":{"lon":10.3951,"lat":63.4305},"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}],"base":"stations","main":{"temp":15.21,"feels_like":15.19,"temp_min":14.99,"temp_max":16.05,"pressure":1014,"humidity":92},"visibility":10000,"wind":{"speed":0.89,"deg":270,"gust":2.68},"clouds":{"all":99},"dt":1692185222,"sys":{"type":2,"id":2046252,"country":"NO","sunrise":1692155757,"sunset":1692214194},"timezone":7200,"id":3133880,"name":"Trondheim","cod":200}"#;

        let json_value: serde_json::Value = serde_json::from_str(json).unwrap();

        let open_weather = OpenWeatherNowcast::try_from(json_value).unwrap();

        assert_eq!(open_weather.dt.timestamp(), 1692185222);
        assert_eq!(open_weather.name, "Trondheim");
        assert_eq!(open_weather.country, "NO");
        assert_eq!(open_weather.lon, 10.3951);
        assert_eq!(open_weather.lat, 63.4305);
        assert_eq!(open_weather.main, "Clouds");
        assert_eq!(open_weather.desc, "overcast clouds");
        assert_eq!(open_weather.clouds, 99);
        assert_eq!(open_weather.wind_speed, 0.89);
        assert_eq!(open_weather.wind_deg, 270);
        assert_eq!(open_weather.visibility, 10000);
        assert_eq!(open_weather.temp, 15.21);
        assert_eq!(open_weather.feels_like, 15.19);
        assert_eq!(open_weather.humidity, 92);
        assert_eq!(open_weather.pressure, 1014);
    }

    #[tokio::test]
    async fn openweathermap_fetch() {
        let client_builder = reqwest::Client::builder();
        static APP_USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
        let location = Coordinates::new(63.4308, 10.4034);
        let nowcast = OpenWeatherNowcast::fetch(&client, &location).await;
        assert!(nowcast.is_ok())
    }
}
