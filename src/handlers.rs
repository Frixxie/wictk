use crate::{
    alerts::{Alert, MetAlert},
    location::{City, LocationQuery, OpenWeatherMapLocation},
    nowcasts::{fetch_met_nowcast, fetch_met_openweathermap, Nowcast},
    utils::InternalApplicationError,
};
use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::Client;
use serde_json::Value;

pub async fn ping() -> &'static str {
    log::info!("GET /status/ping");
    "pong"
}

pub async fn geocoding(client: Client, location: String) -> Option<Vec<OpenWeatherMapLocation>> {
    match client
        .get("https://api.openweathermap.org/geo/1.0/direct")
        .query(&[("q", location)])
        .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
        .send()
        .await
    {
        Ok(result) => match result.json::<Vec<OpenWeatherMapLocation>>().await {
            Ok(res) => Some(res),
            Err(err) => {
                log::error!("Error: {}", err);
                None
            }
        },
        Err(err) => {
            log::error!("Error: {}", err);
            None
        }
    }
}

pub async fn get_geocoding(
    State(client): State<Client>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, InternalApplicationError> {
    log::info!("GET /api/geocoding");
    let res = geocoding(client, query.location).await.ok_or_else(|| {
        log::error!("Failed to get geocoding data from OpenWeatherMap");
        InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
    })?;
    Ok(Json(res))
}

pub async fn alerts(
    State(client): State<Client>,
) -> Result<Json<Vec<Alert>>, InternalApplicationError> {
    log::info!("GET /api/alerts");
    let res: Vec<Alert> = client
        .get("https://api.met.no/weatherapi/metalerts/1.1/.json")
        .send()
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Request to Met.no failed")
        })?
        .json::<Value>()
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Deserialization from Met.no failed")
        })?
        .get("features")
        .ok_or(InternalApplicationError::new(
            "Failed to convert get features value to alert type",
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
    log::info!("GET /api/nowcasts");
    let location = match query {
        LocationQuery::Location(loc_query) => {
            let res = geocoding(client.clone(), loc_query.location)
                .await
                .ok_or_else(|| {
                    InternalApplicationError::new(
                        "Failed to get geocoding data from OpenWeatherMap",
                    )
                })?;
            res.first()
                .ok_or_else(|| {
                    InternalApplicationError::new(
                        "Failed to get geocoding data from OpenWeatherMap",
                    )
                })?
                .location
                .clone()
        }
        LocationQuery::Coordinates(location) => location.try_into().map_err(|_| {
            InternalApplicationError::new("Failed to convert value to location type")
        })?,
    };

    let met_cast_handle = tokio::spawn(fetch_met_nowcast(client.clone(), location.clone()));
    let openweathermap_cast_handle =
        tokio::spawn(fetch_met_openweathermap(client.clone(), location.clone()));

    let met_cast = met_cast_handle
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no nowcast")
        })?
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no nowcast")
        })?;

    let openweathermap = openweathermap_cast_handle
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get OpenWeatherMap nowcast")
        })?
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get OpenWeatherMap nowcast")
        })?;

    let nowcasts: Vec<Nowcast> = vec![met_cast.into(), openweathermap.into()];
    Ok(Json(nowcasts))
}

#[cfg(test)]
mod tests {
    use axum::extract::Query;
    use http::Uri;
    use pretty_assertions::assert_eq;

    use crate::location::{City, CoordinatesAsString, LocationQuery};

    #[test]
    fn parse_location() {
        let uri: Uri = "http://localhost:3000/api/nowcasts?location=Oslo"
            .parse()
            .unwrap();

        let query = Query::<LocationQuery>::try_from_uri(&uri).unwrap();

        assert_eq!(
            query.0,
            LocationQuery::Location(City {
                location: "Oslo".to_string()
            })
        );
    }

    #[test]
    fn parse_coordinates() {
        let uri: Uri = "http://localhost:3000/api/nowcasts?lat=59.91273&lon=10.74609"
            .parse()
            .unwrap();

        let query = Query::<LocationQuery>::try_from_uri(&uri).unwrap();

        assert_eq!(
            query.0,
            LocationQuery::Coordinates(CoordinatesAsString {
                lat: "59.91273".to_string(),
                lon: "10.74609".to_string()
            })
        );
    }
}
