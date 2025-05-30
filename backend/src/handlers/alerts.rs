use crate::AppState;
use axum::{extract::State, Json};
use reqwest::StatusCode;
use tracing::instrument;
use wictk_core::{Alert, MetAlert};

use super::error::ApplicationError;

pub type Alerts = Vec<Alert>;

#[instrument]
pub async fn alerts(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<Alert>>, ApplicationError> {
    match app_state.alert_cache.get("met_alerts").await {
        Some(alerts) => return Ok(Json(alerts.clone())),
        None => {
            let alerts = MetAlert::fetch(app_state.client.clone())
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
                .insert("met_alerts".to_string(), alerts.clone())
                .await;
            Ok(Json(alerts))
        }
    }
}
