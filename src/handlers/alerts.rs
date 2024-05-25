use std::time::Duration;

use crate::{
    alerts::{Alert, AlertFetcher, MetAlert},
    cache::TimedCache,
    locations::{City, OpenWeatherMapLocation},
    AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use tokio::time::Instant;

use super::error::InternalApplicationError;

pub type Alerts = Vec<Alert>;

pub async fn alerts(
    State(app_state): State<AppState>,
    Query(query): Query<City>,
) -> Result<Json<Vec<Alert>>, InternalApplicationError> {
    log::info!("GET /api/alerts");
    let res = OpenWeatherMapLocation::fetch(&app_state.client, &query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
    let alerts = app_state.cache.get("alert".to_string()).await;
    match alerts {
        Some(alerts) => Ok(Json(alerts)),
        None => {
            let alerts = MetAlert::fetch(
                app_state.client.clone(),
                res.first().unwrap().clone().location,
            )
            .await
            .map_err(|err| {
                log::error!("Error {}", err);
                InternalApplicationError::new("Failed to get Met.no alerts")
            })?;
            app_state
                .cache
                .set(
                    "alert".to_string(),
                    alerts.clone(),
                    Instant::now() + Duration::from_secs(300),
                )
                .await;
            Ok(Json(alerts))
        }
    }
}
