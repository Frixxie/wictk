use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::Client;

use crate::{
    alerts::{Alert, AlertFetcher, MetAlert},
    handlers::location::OpenWeatherMapLocation,
};

use super::{error::InternalApplicationError, location::City};

pub async fn alerts(
    State(client): State<Client>,
    Query(query): Query<City>,
) -> Result<Json<Vec<Alert>>, InternalApplicationError> {
    log::info!("GET /api/alerts");
    let res = OpenWeatherMapLocation::fetch(&client, &query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
    let alerts = MetAlert::fetch(client.clone(), res.first().unwrap().clone().location)
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no alerts")
        })?;
    Ok(Json(alerts))
}
