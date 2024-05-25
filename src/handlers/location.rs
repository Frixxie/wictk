use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    locations::{City, OpenWeatherMapLocation},
    AppState,
};

use super::error::InternalApplicationError;

pub async fn geocoding(
    State(app_state): State<AppState>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, InternalApplicationError> {
    let res = OpenWeatherMapLocation::fetch(&app_state.client, &query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
    Ok(Json(res))
}
