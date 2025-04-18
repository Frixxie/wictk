use axum::{
    extract::{Query, State},
    Json,
};
use redact::Secret;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing::instrument;
use wictk_core::{
    City, Coordinates, CoordinatesAsString, MetNowcast, Nowcast, OpenWeatherMapLocation, OpenWeatherNowcast,
};

use crate::{cache::Cache, AppState};

use super::{error::ApplicationError, location::lookup_location};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LocationQuery {
    Location(City),
    Coordinates(CoordinatesAsString),
}

pub async fn find_location(
    location_query: LocationQuery,
    client: &Client,
    location_cache: &Cache<String, Option<OpenWeatherMapLocation>>,
    apikey: &Secret<String>,
) -> anyhow::Result<Coordinates> {
    match location_query {
        LocationQuery::Location(location) => {
            let location =
                lookup_location(client, &location.location, location_cache, apikey).await;
            match location {
                Ok(location) => Ok(location.location),
                Err(err) => Err(err.into()),
            }
        }
        LocationQuery::Coordinates(cords_as_string) => {
            let cords = cords_as_string.try_into()?;
            Ok(cords)
        }
    }
}

#[instrument]
pub async fn nowcast_met(
    app_state: State<AppState>,
    Query(location): Query<LocationQuery>,
) -> Result<Json<Nowcast>, ApplicationError> {
    let location = find_location(
        location,
        &app_state.client,
        &app_state.location_cache,
        &app_state.openweathermap_apikey,
    )
    .await
    .map_err(|err| {
        error!("Error finding location: {:?}", err);
        ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let nowcast = MetNowcast::fetch(&app_state.client, &location)
        .await
        .map_err(|err| {
            error!("Error fetching Met.no nowcast: {:?}", err);
            ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    Ok(Json(nowcast))
}

#[instrument]
pub async fn nowcast_openweathermap(
    app_state: State<AppState>,
    Query(location): Query<LocationQuery>,
) -> Result<Json<Nowcast>, ApplicationError> {
    let location = find_location(
        location,
        &app_state.client,
        &app_state.location_cache,
        &app_state.openweathermap_apikey,
    )
    .await
    .map_err(|err| {
        error!("Error finding location: {:?}", err);
        ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let nowcast = OpenWeatherNowcast::fetch(
        &app_state.client,
        &location,
        &app_state.openweathermap_apikey,
    )
    .await
    .map_err(|err| {
        error!("Error fetching from OpenWeatherMap.com nowcast: {:?}", err);
        ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    Ok(Json(nowcast))
}

#[instrument]
pub async fn nowcasts(
    app_state: State<AppState>,
    Query(location): Query<LocationQuery>,
) -> Result<Json<Vec<Nowcast>>, ApplicationError> {
    let location = find_location(
        location,
        &app_state.client,
        &app_state.location_cache,
        &app_state.openweathermap_apikey,
    )
    .await
    .map_err(|err| {
        error!("Error finding location: {:?}", err);
        ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let open_nowcast = OpenWeatherNowcast::fetch(
        &app_state.client,
        &location,
        &app_state.openweathermap_apikey,
    )
    .await
    .map_err(|err| {
        error!("Error fetching from OpenWeatherMap.com nowcast: {:?}", err);
        ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let met_nowcast = MetNowcast::fetch(&app_state.client, &location)
        .await
        .map_err(|err| {
            error!("Error fetching Met.no nowcast: {:?}", err);
            ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    Ok(Json(vec![met_nowcast, open_nowcast]))
}
