use std::time::Duration;

use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::StatusCode;
use tokio::time::Instant;

use crate::{
    cache::{Cache, TimedCache},
    locations::{City, OpenWeatherMapLocation},
    AppState,
};

use super::error::ApplicationError;

async fn populate_location_cache(
    location: &str,
    locations: Option<Vec<OpenWeatherMapLocation>>,
    loc_cache: &Cache<String, Option<OpenWeatherMapLocation>>,
) -> Option<OpenWeatherMapLocation> {
    match locations {
        Some(locs) => match locs.first() {
            Some(loc) => {
                loc_cache
                    .set(
                        location.to_string(),
                        Some(loc.clone()),
                        Instant::now() + Duration::from_secs(300),
                    )
                    .await;
                Some(loc.clone())
            }
            None => {
                loc_cache
                    .set(
                        location.to_string(),
                        None,
                        Instant::now() + Duration::from_secs(300),
                    )
                    .await;
                None
            }
        },
        None => {
            loc_cache
                .set(
                    location.to_string(),
                    None,
                    Instant::now() + Duration::from_secs(300),
                )
                .await;
            None
        }
    }
}

pub async fn lookup_location(
    client: &reqwest::Client,
    location: &str,
    loc_cache: &Cache<String, Option<OpenWeatherMapLocation>>,
    apikey: &str,
) -> Result<OpenWeatherMapLocation, ApplicationError> {
    match loc_cache.get(location.to_string()).await {
        Some(location) => match location {
            Some(loc) => Ok(loc),
            None => Err(ApplicationError::new(
                "Not found in cache",
                StatusCode::NOT_FOUND,
            )),
        },
        None => {
            let res = OpenWeatherMapLocation::fetch(client, location, apikey).await;
            match populate_location_cache(location, res.clone(), loc_cache).await {
                Some(loc) => Ok(loc),
                None => Err(ApplicationError::new(
                    "Location not found",
                    StatusCode::NOT_FOUND,
                )),
            }
        }
    }
}

pub async fn geocoding(
    State(app_state): State<AppState>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, ApplicationError> {
    let res = OpenWeatherMapLocation::fetch(
        &app_state.client,
        &query.location,
        &app_state.openweathermap_apikey,
    )
    .await
    .ok_or_else(|| {
        tracing::error!("Failed to get geocoding data from OpenWeatherMap");
        ApplicationError::new(
            "Failed to get geocoding data from OpenWeatherMap",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    Ok(Json(res))
}
