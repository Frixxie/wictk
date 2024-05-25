use std::time::Duration;

use crate::{
    alerts::{Alert, AlertFetcher, MetAlert},
    cache::TimedCache,
    location::{City, Coordinates, LocationQuery, OpenWeatherMapLocation},
    nowcasts::{MetNowcast, Nowcast, NowcastFetcher, OpenWeatherNowcast},
    utils::InternalApplicationError,
    AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use log::error;
use tokio::time::Instant;

pub async fn ping() -> &'static str {
    log::info!("GET /status/ping");
    "pong"
}

pub async fn geocoding(
    State(app_state): State<AppState>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, InternalApplicationError> {
    log::info!("GET /api/geocoding");
    let res = OpenWeatherMapLocation::fetch(app_state.client, query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
    Ok(Json(res))
}

pub type Alerts = Vec<Alert>;

pub async fn alerts(
    State(app_state): State<AppState>,
) -> Result<Json<Alerts>, InternalApplicationError> {
    log::info!("GET /api/alerts");
    let alerts = app_state.cache.get("alert".to_string()).await;
    match alerts {
        Some(alerts) => Ok(Json(alerts)),
        None => {
            let alerts = MetAlert::fetch(
                app_state.client.clone(),
                Coordinates::new(59.91273, 10.74609),
            )
            .await
            .map_err(|err| {
                log::error!("Error {}", err);
                InternalApplicationError::new("Failed to get Met.no alerts")
            })?;
            app_state
                .cache
                .set(
                    "alert".to_string(),
                    alerts.clone(),
                    Instant::now() + Duration::from_secs(300),
                )
                .await;
            Ok(Json(alerts))
        }
    }
}

pub async fn nowcasts(
    State(app_state): State<AppState>,
    Query(query): Query<LocationQuery>,
) -> Result<Json<Vec<Nowcast>>, InternalApplicationError> {
    log::info!("GET /api/nowcasts");
    let location = match query {
        LocationQuery::Location(loc_query) => {
            let res = OpenWeatherMapLocation::fetch(app_state.client.clone(), loc_query.location)
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

    let met_cast_handle = tokio::spawn(MetNowcast::fetch(
        app_state.client.clone(),
        location.clone(),
    ));
    let openweathermap_cast_handle = tokio::spawn(OpenWeatherNowcast::fetch(
        app_state.client.clone(),
        location.clone(),
    ));

    let met_cast = met_cast_handle
        .await
        .map_err(|err| {
            error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no nowcast")
        })
        .and_then(|res| {
            res.map_err(|err| {
                log::error!("Error {}", err);
                InternalApplicationError::new("Failed to get Met.no nowcast")
            })
        });

    let openweathermap = openweathermap_cast_handle
        .await
        .map_err(|err| {
            error!("Error {}", err);
            InternalApplicationError::new("Failed to get OpenWeatherMap nowcast")
        })
        .and_then(|res| {
            res.map_err(|err| {
                log::error!("Error {}", err);
                InternalApplicationError::new("Failed to get OpenWeatherMap nowcast")
            })
        });

    let nowcasts: Vec<Nowcast> = vec![met_cast, openweathermap]
        .into_iter()
        .filter_map(|res| res.ok())
        .collect();

    Ok(Json(nowcasts))
}

#[cfg(test)]
mod tests {
    use axum::extract::Query;
    use axum::http::Uri;
    use pretty_assertions::assert_eq;

    use crate::{
        cache,
        location::{City, CoordinatesAsString, LocationQuery},
        AppState,
    };

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

    #[tokio::test]
    async fn get_geocoding() {
        let client = reqwest::Client::new();
        let cache = cache::Cache::new();
        let appstate = AppState::new(client, cache);
        let res = super::geocoding(
            axum::extract::State(appstate),
            axum::extract::Query(super::City {
                location: "Oslo".to_string(),
            }),
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_alerts() {
        let client_builder = reqwest::Client::builder();
        static APP_USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
        let cache = cache::Cache::new();
        let appstate = AppState::new(client, cache);
        let res = super::alerts(axum::extract::State(appstate)).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn get_nowcasts() {
        let client_builder = reqwest::Client::builder();
        static APP_USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
        let cache = cache::Cache::new();
        let appstate = AppState::new(client, cache);
        let res = super::nowcasts(
            axum::extract::State(appstate),
            axum::extract::Query(super::LocationQuery::Coordinates(CoordinatesAsString {
                lat: "59.91273".to_string(),
                lon: "10.74609".to_string(),
            })),
        )
        .await;
        assert!(res.is_ok());
    }
}
