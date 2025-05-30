use axum::{extract::State, Json};
use reqwest::StatusCode;
use tracing::{error, instrument};
use wictk_core::Lightning;

use crate::AppState;

use super::error::ApplicationError;

#[instrument]
pub async fn get_recent_lightning(
    app_state: State<AppState>,
) -> Result<Json<Vec<Lightning>>, ApplicationError> {
    let lightning = Lightning::find_ligntning(
        &app_state.client,
        "https://www.yr.no/api/v0/lightning-events?fromHours=24",
    )
    .await
    .map_err(|err| {
        error!("Error fetching lightning data: {:?}", err);
        ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    Ok(Json(lightning))
}
