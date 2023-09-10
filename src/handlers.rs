use crate::{
    utils::InternalApplicationError,
    weather::{
        Alert, Location, MetAlert, MetNowcast, Nowcast, OpenWeatherLocationEntry,
        OpenWeatherNowcast,
    },
};
use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub async fn geocoding(client: Client, location: String) -> Option<Vec<OpenWeatherLocationEntry>> {
    client
        .get("http://api.openweathermap.org/geo/1.0/direct")
        .query(&[("q", location)])
        .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
        .send()
        .await
        .ok()?
        .json::<Vec<OpenWeatherLocationEntry>>()
        .await
        .ok()
}

#[derive(Serialize, Deserialize)]
pub struct LocationQuery {
    pub location: Option<String>,
    #[serde(flatten)]
    pub lon_lat: Option<Location>,
}

pub async fn get_geocoding(
    State(client): State<Client>,
    Query(query): Query<LocationQuery>,
) -> Result<Json<Vec<OpenWeatherLocationEntry>>, InternalApplicationError> {
    let location = query
        .location
        .ok_or_else(|| InternalApplicationError::new("No location provided in query string"))?;

    let res = geocoding(client, location).await.ok_or_else(|| {
        InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
    })?;
    Ok(Json(res))
}

pub async fn alerts(
    State(client): State<Client>,
) -> Result<Json<Vec<Alert>>, InternalApplicationError> {
    let res: Vec<Alert> = client
        .get("https://api.met.no/weatherapi/metalerts/1.1/.json")
        .send()
        .await
        .map_err(|_| InternalApplicationError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| InternalApplicationError::new("Deserialization failed"))?
        .get("features")
        .ok_or(InternalApplicationError::new(
            "Failed to convert value to alert type",
        ))?
        .as_array()
        .ok_or(InternalApplicationError::new(
            "Failed to convert value to alert type",
        ))?
        .iter()
        .filter_map(|alert| MetAlert::try_from(alert.clone()).ok())
        .map(|alert| alert.into())
        .collect();
    Ok(Json(res))
}

pub async fn nowcasts(
    State(client): State<Client>,
    Query(query): Query<LocationQuery>,
) -> Result<Json<Vec<Nowcast>>, InternalApplicationError> {
    let location = if let Some(location) = query.lon_lat {
        location
    } else {
        let location = query
            .location
            .ok_or_else(|| InternalApplicationError::new("No location provided in query string"))?;
        let res = geocoding(client.clone(), location).await.ok_or_else(|| {
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
        res.first()
            .ok_or_else(|| InternalApplicationError::new("No location found"))?
            .location
            .clone()
    };
    let met_cast: MetNowcast = client
        .get("https://api.met.no/weatherapi/nowcast/2.0/complete")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .send()
        .await
        .map_err(|_| InternalApplicationError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| InternalApplicationError::new("Deserialization failed"))?
        .try_into()
        .map_err(|_| InternalApplicationError::new("Failed to convert value to nowcast type"))?;
    let openweathermap: OpenWeatherNowcast = client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
        .send()
        .await
        .map_err(|_| InternalApplicationError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| InternalApplicationError::new("Deserialization failed"))?
        .try_into()
        .map_err(|_| InternalApplicationError::new("Failed to convert value to nowcast type"))?;
    let nowcasts: Vec<Nowcast> = vec![met_cast.into(), openweathermap.into()];
    Ok(Json(nowcasts))
}
