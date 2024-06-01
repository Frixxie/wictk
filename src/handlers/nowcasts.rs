use std::time::Duration;

use axum::{
    extract::{Query, State},
    Json,
};
use log::error;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::{
    cache::{Cache, TimedCache},
    locations::{City, Coordinates, CoordinatesAsString, OpenWeatherMapLocation},
    nowcasts::{MetNowcast, Nowcast, NowcastFetcher, OpenWeatherNowcast},
    AppState,
};

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
) -> anyhow::Result<Coordinates> {
    match location_query {
        LocationQuery::Location(location) => {
            let location = lookup_location(client, &location.location, location_cache).await;
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

#[derive(Serialize, Deserialize)]
pub enum NowcastProvider {
    Met,
    OpenWeatherMap,
}

#[derive(Serialize, Deserialize)]
pub struct ProviderQuery {
    pub provider: Option<NowcastProvider>,
}

async fn fetch_from_provider<T>(
    client: &Client,
    location: &Coordinates,
    provider_name: &str,
    nowcast_cache: &Cache<String, Nowcast>,
) -> Result<Nowcast, ApplicationError>
where
    T: NowcastFetcher,
{
    let nowcast = nowcast_cache
        .get(format!("{}-{}", location.to_string(), provider_name))
        .await;
    match nowcast {
        Some(nowcast) => Ok(nowcast),
        None => {
            let res = T::fetch(client, location).await.map_err(|err| {
                error!("Error {}", err);
                ApplicationError::new("Failed to get nowcast", StatusCode::INTERNAL_SERVER_ERROR)
            })?;
            nowcast_cache
                .set(
                    location.to_string(),
                    res.clone(),
                    Instant::now() + Duration::from_secs(300),
                )
                .await;
            Ok(res)
        }
    }
}

pub async fn nowcasts(
    State(app_state): State<AppState>,
    Query(provider_query): Query<ProviderQuery>,
    Query(location_query): Query<LocationQuery>,
) -> Result<Json<Vec<Nowcast>>, ApplicationError> {
    let location = find_location(location_query, &app_state.client, &app_state.location_cache)
        .await
        .map_err(|err| {
            error!("Error {}", err);
            ApplicationError::new(
                "Failed to get location data",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let casts = match provider_query.provider {
        Some(provider) => match provider {
            NowcastProvider::Met => {
                vec![
                    fetch_from_provider::<MetNowcast>(
                        &app_state.client,
                        &location,
                        "MET",
                        &app_state.nowcast_cache,
                    )
                    .await,
                ]
            }
            NowcastProvider::OpenWeatherMap => {
                vec![
                    fetch_from_provider::<OpenWeatherNowcast>(
                        &app_state.client,
                        &location,
                        "OWM",
                        &app_state.nowcast_cache,
                    )
                    .await,
                ]
            }
        },
        None => vec![
            fetch_from_provider::<MetNowcast>(
                &app_state.client,
                &location,
                "MET",
                &app_state.nowcast_cache,
            )
            .await,
            fetch_from_provider::<OpenWeatherNowcast>(
                &app_state.client,
                &location,
                "OWM",
                &app_state.nowcast_cache,
            )
            .await,
        ],
    };

    let nowcasts: Vec<Nowcast> = casts.into_iter().filter_map(|res| res.ok()).collect();

    Ok(Json(nowcasts))
}
