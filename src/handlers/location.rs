use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::Client;

use crate::locations::{City, OpenWeatherMapLocation};

use super::error::InternalApplicationError;

pub async fn geocoding(
    State(client): State<Client>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, InternalApplicationError> {
    log::info!("GET /api/geocoding");
    let res = OpenWeatherMapLocation::fetch(&client, &query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
    Ok(Json(res))
}
