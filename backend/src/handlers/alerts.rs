use crate::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::StatusCode;
use tracing::instrument;
use wictk_core::{Alert, City, MetAlert};

use super::{error::ApplicationError, location::lookup_location};

pub type Alerts = Vec<Alert>;

#[instrument]
pub async fn alerts(
    State(app_state): State<AppState>,
    Query(query): Query<City>,
) -> Result<Json<Vec<Alert>>, ApplicationError> {
    let location = lookup_location(
        &app_state.client,
        &query.location,
        &app_state.location_cache,
        &app_state.openweathermap_apikey,
    )
    .await?;

    let alerts = app_state.alert_cache.get(&location.name).await;
    match alerts {
        Some(alerts) => Ok(Json(alerts)),
        None => {
            let alerts = MetAlert::fetch(app_state.client.clone(), location.location)
                .await
                .map_err(|err| {
                    tracing::error!("Error {}", err);
                    ApplicationError::new(
                        "Failed to get Met.no alerts",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?;
            app_state
                .alert_cache
                .insert(location.name.to_string(), alerts.clone())
                .await;
            Ok(Json(alerts))
        }
    }
}
