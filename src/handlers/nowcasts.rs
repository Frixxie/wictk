use axum::{
    extract::{Query, State},
    Json,
};
use log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{City, Coordinates, CoordinatesAsString, OpenWeatherMapLocation},
    nowcasts::{MetNowcast, Nowcast, NowcastFetcher, OpenWeatherNowcast},
};

use super::error::InternalApplicationError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LocationQuery {
    Location(City),
    Coordinates(CoordinatesAsString),
}

pub async fn find_location(
    location_query: LocationQuery,
    client: &Client,
) -> anyhow::Result<Coordinates> {
    match location_query {
        LocationQuery::Location(location) => {
            let res = OpenWeatherMapLocation::fetch(client, &location.location).await;
            let location = res.ok_or_else(|| {
                InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
            })?;
            Ok(location.first().unwrap().location.clone())
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
) -> Result<Nowcast, InternalApplicationError>
where
    T: NowcastFetcher,
{
    T::fetch(client, location).await.map_err(|err| {
        error!("Error {}", err);
        InternalApplicationError::new("Failed to get nowcast")
    })
}

pub async fn nowcasts(
    State(client): State<Client>,
    Query(provider_query): Query<ProviderQuery>,
    Query(location_query): Query<LocationQuery>,
) -> Result<Json<Vec<Nowcast>>, InternalApplicationError> {
    log::info!("GET /api/nowcasts");
    let location = find_location(location_query, &client)
        .await
        .map_err(|err| {
            error!("Error {}", err);
            InternalApplicationError::new("Failed to get location data")
        })?;

    let casts = match provider_query.provider {
        Some(provider) => match provider {
            NowcastProvider::Met => {
                vec![fetch_from_provider::<MetNowcast>(&client, &location).await]
            }
            NowcastProvider::OpenWeatherMap => {
                vec![fetch_from_provider::<OpenWeatherNowcast>(&client, &location).await]
            }
        },
        None => vec![
            fetch_from_provider::<MetNowcast>(&client, &location).await,
            fetch_from_provider::<OpenWeatherNowcast>(&client, &location).await,
        ],
    };

    let nowcasts: Vec<Nowcast> = casts.into_iter().filter_map(|res| res.ok()).collect();

    Ok(Json(nowcasts))
}
