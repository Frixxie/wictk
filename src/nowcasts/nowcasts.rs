use std::error::Error;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::location::Coordinates;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Nowcast {
    Met(MetNowcast),
    OpenWeather(OpenWeatherNowcast),
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

impl From<MetNowcast> for Nowcast {
    fn from(met: MetNowcast) -> Self {
        Self::Met(met)
    }
}

impl From<OpenWeatherNowcast> for Nowcast {
    fn from(open_weather: OpenWeatherNowcast) -> Self {
        Self::OpenWeather(open_weather)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetNowcast {
    pub location: Coordinates,
    pub time: String,
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
            time: time.to_string(),
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

pub async fn fetch_met_nowcast(
    client: Client,
    location: Coordinates,
) -> Result<MetNowcast, NowcastError> {
    let met_cast: MetNowcast = client
        .get("https://api.met.no/weatherapi/nowcast/2.0/complete")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .send()
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            NowcastError::new("Request to Met.no failed")
        })?
        .json::<Value>()
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            NowcastError::new("Deserialization from Met.no failed")
        })?
        .try_into()
        .map_err(|err| {
            log::error!("Error {}", err);
            NowcastError::new("Failed to convert from met value into nowcast type")
        })?;
    Ok(met_cast)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenWeatherNowcast {
    dt: u32,
    name: String,
    country: String,
    lon: f32,
    lat: f32,
    main: String,
    desc: String,
    clouds: u32,
    wind_speed: f32,
    wind_deg: i32,
    visibility: i32,
    temp: f32,
    feels_like: f32,
    humidity: u32,
    pressure: u32,
}

impl TryFrom<serde_json::Value> for OpenWeatherNowcast {
    type Error = NowcastError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let dt = value["dt"]
            .as_u64()
            .ok_or(NowcastError::new("Could not find dt"))? as u32;
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

pub async fn fetch_met_openweathermap(
    client: Client,
    location: Coordinates,
) -> Result<OpenWeatherNowcast, NowcastError> {
    let openweathermap: OpenWeatherNowcast = client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
        .send()
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            NowcastError::new("Request to OpenWeatherMap failed")
        })?
        .json::<Value>()
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            NowcastError::new("Deserialization from OpenWeatherMap failed")
        })?
        .try_into()
        .map_err(|err| {
            log::error!("Error {}", err);
            NowcastError::new("Failed to convert OpenWeatherMap value into nowcast type")
        })?;
    Ok(openweathermap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn met_test_from_value() {
        let json = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[10.4034,63.4308,0]},"properties":{"meta":{"updated_at":"2023-08-14T18:16:07Z","units":{"air_temperature":"celsius","precipitation_amount":"mm","precipitation_rate":"mm/h","relative_humidity":"%","wind_from_direction":"degrees","wind_speed":"m/s","wind_speed_of_gust":"m/s"},"radar_coverage":"ok"},"timeseries":[{"time":"2023-08-14T18:15:00Z","data":{"instant":{"details":{"air_temperature":17.7,"precipitation_rate":0.0,"relative_humidity":80.5,"wind_from_direction":294.4,"wind_speed":2.7,"wind_speed_of_gust":6.1}},"next_1_hours":{"summary":{"symbol_code":"cloudy"},"details":{"precipitation_amount":0.0}}}},{"time":"2023-08-14T18:20:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:25:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:30:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:35:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:40:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:45:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:50:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T18:55:00Z","data":{"instant":{"details":{"precipitation_rate":0.0}}}},{"time":"2023-08-14T19:00:00Z","data":{"instant":{"details":{"precipitation_rate":0.2}}}},{"time":"2023-08-14T19:05:00Z","data":{"instant":{"details":{"precipitation_rate":0.5}}}},{"time":"2023-08-14T19:10:00Z","data":{"instant":{"details":{"precipitation_rate":0.7}}}},{"time":"2023-08-14T19:15:00Z","data":{"instant":{"details":{"precipitation_rate":0.9}}}},{"time":"2023-08-14T19:20:00Z","data":{"instant":{"details":{"precipitation_rate":1.1}}}},{"time":"2023-08-14T19:25:00Z","data":{"instant":{"details":{"precipitation_rate":1.4}}}},{"time":"2023-08-14T19:30:00Z","data":{"instant":{"details":{"precipitation_rate":1.6}}}},{"time":"2023-08-14T19:35:00Z","data":{"instant":{"details":{"precipitation_rate":1.8}}}},{"time":"2023-08-14T19:40:00Z","data":{"instant":{"details":{"precipitation_rate":1.8}}}},{"time":"2023-08-14T19:45:00Z","data":{"instant":{"details":{"precipitation_rate":1.9}}}},{"time":"2023-08-14T19:50:00Z","data":{"instant":{"details":{"precipitation_rate":1.9}}}},{"time":"2023-08-14T19:55:00Z","data":{"instant":{"details":{"precipitation_rate":1.9}}}},{"time":"2023-08-14T20:00:00Z","data":{"instant":{"details":{"precipitation_rate":2.0}}}},{"time":"2023-08-14T20:05:00Z","data":{"instant":{"details":{"precipitation_rate":2.5}}}}]}}"#;

        let json_value: serde_json::Value = serde_json::from_str(json).unwrap();

        let met = MetNowcast::try_from(json_value).unwrap();

        assert_eq!(met.location.lat, 10.4034);
        assert_eq!(met.location.lon, 63.4308);
        assert_eq!(met.time, "2023-08-14T18:16:07Z");
        assert_eq!(met.description, "cloudy");
        assert_eq!(met.air_temperature, 17.7);
        assert_eq!(met.relative_humidity, 80.5);
        assert_eq!(met.precipitation_amount, 0.0);
        assert_eq!(met.wind_speed, 2.7);
        assert_eq!(met.wind_speed_gust, 6.1);
        assert_eq!(met.wind_from_direction, 294.4);
    }

    #[test]
    fn open_weathermap_from_value() {
        let json = r#"{"coord":{"lon":10.3951,"lat":63.4305},"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}],"base":"stations","main":{"temp":15.21,"feels_like":15.19,"temp_min":14.99,"temp_max":16.05,"pressure":1014,"humidity":92},"visibility":10000,"wind":{"speed":0.89,"deg":270,"gust":2.68},"clouds":{"all":99},"dt":1692185222,"sys":{"type":2,"id":2046252,"country":"NO","sunrise":1692155757,"sunset":1692214194},"timezone":7200,"id":3133880,"name":"Trondheim","cod":200}"#;

        let json_value: serde_json::Value = serde_json::from_str(json).unwrap();

        let open_weather = OpenWeatherNowcast::try_from(json_value).unwrap();

        assert_eq!(open_weather.dt, 1692185222);
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
}
