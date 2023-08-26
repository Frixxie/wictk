mod weather;

use core::fmt;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router, Server,
};
use hyper::StatusCode;
use reqwest::Client;
use serde_json::Value;
use weather::{Alert, Location, MetAlert, MetNowcast, Nowcast, OpenWeatherNowcast};

#[derive(Debug)]
pub struct InternalApplicationError {
    message: String,
}

impl Display for InternalApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InternalApplicationError {}

impl InternalApplicationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for InternalApplicationError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            self.message().to_string(),
        )
            .into_response()
    }
}

async fn alerts(
    State(client): State<Client>,
) -> Result<Json<Vec<Alert>>, InternalApplicationError> {
    let res: Vec<Alert> = client
        .get("https://api.met.no/weatherapi/metalerts/1.1/.json")
        .send()
        .await
        .map_err(|_| InternalApplicationError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| InternalApplicationError::new("Deserialization failed"))?
        .get("features")
        .ok_or(InternalApplicationError::new(
            "Failed to convert value to alert type",
        ))?
        .as_array()
        .ok_or(InternalApplicationError::new(
            "Failed to convert value to alert type",
        ))?
        .iter()
        .filter_map(|alert| MetAlert::try_from(alert.clone()).ok())
        .map(|alert| alert.into())
        .collect();
    Ok(Json(res))
}

async fn nowcasts(
    State(client): State<Client>,
    Query(location): Query<Location>,
) -> Result<Json<Vec<Nowcast>>, InternalApplicationError> {
    let met_cast: MetNowcast = client
        .get("https://api.met.no/weatherapi/nowcast/2.0/complete")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .send()
        .await
        .map_err(|_| InternalApplicationError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| InternalApplicationError::new("Deserialization failed"))?
        .try_into()
        .map_err(|_| InternalApplicationError::new("Failed to convert value to nowcast type"))?;
    let openweathermap: OpenWeatherNowcast = client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
        .send()
        .await
        .map_err(|_| InternalApplicationError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| InternalApplicationError::new("Deserialization failed"))?
        .try_into()
        .map_err(|_| InternalApplicationError::new("Failed to convert value to nowcast type"))?;
    let nowcasts: Vec<Nowcast> = vec![met_cast.into(), openweathermap.into()];
    Ok(Json(nowcasts))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_builder = reqwest::Client::builder();
    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        " ",
        env!("CARGO_PKG_HOMEPAGE"),
    );
    let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();

    let api = Router::new()
        .route("/alerts", get(alerts))
        .route("/nowcasts", get(nowcasts))
        .with_state(client);

    let app = Router::new().nest("/api", api);

    Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
