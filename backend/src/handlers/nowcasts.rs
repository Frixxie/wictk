use axum::{
    extract::{Query, State},
    Json,
};
use moka::future::Cache;
use redact::Secret;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing::instrument;
use wictk_core::{
    City, Coordinates, CoordinatesAsString, MetNowcast, Nowcast, OpenWeatherMapLocation,
    OpenWeatherNowcast,
};

use crate::AppState;

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
    location_cache: &Cache<String, OpenWeatherMapLocation>,
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

    match app_state
        .nowcast_cache
        .get(&format!("met_{}", location.to_string()))
        .await
    {
        Some(nowcast) => return Ok(Json(nowcast)),
        None => {
            let nowcast = MetNowcast::fetch(&app_state.client, &location)
                .await
                .map_err(|err| {
                    error!("Error fetching Met.no nowcast: {:?}", err);
                    ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                })?;
            app_state
                .nowcast_cache
                .insert(format!("met_{}", location.to_string()), nowcast.clone())
                .await;
            Ok(Json(nowcast))
        }
    }
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

    match app_state
        .nowcast_cache
        .get(&format!("open_{}", location.to_string()))
        .await
    {
        Some(nowcast) => return Ok(Json(nowcast)),
        None => {
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
            app_state
                .nowcast_cache
                .insert(format!("open_{}", location.to_string()), nowcast.clone())
                .await;
            Ok(Json(nowcast))
        }
    }
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

    let open_nowcast = match app_state
        .nowcast_cache
        .get(&format!("open_{}", location.to_string()))
        .await
    {
        Some(nowcast) => nowcast,
        None => {
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
            app_state
                .nowcast_cache
                .insert(
                    format!("open_{}", location.to_string()),
                    open_nowcast.clone(),
                )
                .await;
            open_nowcast
        }
    };

    let met_nowcast = match app_state
        .nowcast_cache
        .get(&format!("met_{}", location.to_string()))
        .await
    {
        Some(nowcast) => nowcast,
        None => {
            let met_nowcast = MetNowcast::fetch(&app_state.client, &location)
                .await
                .map_err(|err| {
                    error!("Error fetching Met.no nowcast: {:?}", err);
                    ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                })?;
            app_state
                .nowcast_cache
                .insert(format!("met_{}", location.to_string()), met_nowcast.clone())
                .await;
            met_nowcast
        }
    };
    Ok(Json(vec![met_nowcast, open_nowcast]))
}
