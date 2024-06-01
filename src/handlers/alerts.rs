use std::time::Duration;

use crate::{
    alerts::{Alert, AlertFetcher, MetAlert},
    cache::TimedCache,
    locations::City,
    AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::StatusCode;
use tokio::time::Instant;

use super::{error::ApplicationError, location::lookup_location};

pub type Alerts = Vec<Alert>;

pub async fn alerts(
    State(app_state): State<AppState>,
    Query(query): Query<City>,
) -> Result<Json<Vec<Alert>>, ApplicationError> {
    let location = lookup_location(
        &app_state.client,
        &query.location,
        &app_state.location_cache,
    )
    .await?;

    let alerts = app_state.alert_cache.get(location.name.clone()).await;
    match alerts {
        Some(alerts) => Ok(Json(alerts)),
        None => {
            let alerts = MetAlert::fetch(app_state.client.clone(), location.location)
                .await
                .map_err(|err| {
                    log::error!("Error {}", err);
                    ApplicationError::new(
                        "Failed to get Met.no alerts",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?;
            app_state
                .alert_cache
                .set(
                    location.name.to_string(),
                    alerts.clone(),
                    Instant::now() + Duration::from_secs(300),
                )
                .await;
            Ok(Json(alerts))
        }
    }
}
