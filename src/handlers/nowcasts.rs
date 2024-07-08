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
    nowcasts::{MetNowcast, Nowcast, NowcastFetcher, OpenWeatherNowcast, SimpleNowcast},
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
    nowcast_cache: &Cache<String, Option<Nowcast>>,
) -> Result<Nowcast, ApplicationError>
where
    T: NowcastFetcher,
{
    let nowcast = nowcast_cache
        .get(format!("{}-{}", location.to_string(), provider_name))
        .await;
    match nowcast {
        Some(nowcast) => {
            let res = nowcast
                .ok_or_else(|| ApplicationError::new("{} Not in cache", StatusCode::NOT_FOUND))?;
            Ok(res)
        }
        None => {
            let res = T::fetch(client, location).await.map_err(|err| {
                error!("Error {}", err);
                ApplicationError::new("Failed to get nowcast", StatusCode::INTERNAL_SERVER_ERROR)
            });
            match res {
                Ok(r) => {
                    nowcast_cache
                        .set(
                            format!("{}-{}", location.to_string(), provider_name),
                            Some(r.clone()),
                            Instant::now() + Duration::from_secs(300),
                        )
                        .await;
                    Ok(r)
                }
                Err(e) => {
                    nowcast_cache
                        .set(
                            format!("{}-{}", location.to_string(), provider_name),
                            None,
                            Instant::now() + Duration::from_secs(300),
                        )
                        .await;
                    Err(e)
                }
            }
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
        None => {
            let res = vec![
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
            ];
            res.into_iter()
                .map(|res| {
                    res.and_then(|r| match r {
                        Nowcast::Met(r) => Ok(Nowcast::Simple(Into::<SimpleNowcast>::into(r))),
                        Nowcast::OpenWeather(r) => {
                            Ok(Nowcast::Simple(Into::<SimpleNowcast>::into(r)))
                        }
                        Nowcast::Simple(_) => panic!("Should not be able to get simple provider"),
                    })
                })
                .collect()
        }
    };

    let nowcasts: Vec<Nowcast> = casts.into_iter().filter_map(|res| res.ok()).collect();

    Ok(Json(nowcasts))
}
