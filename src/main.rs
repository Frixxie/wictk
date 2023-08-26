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
pub struct AppError {
    message: String,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AppError {}

impl AppError {
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
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            self.message().to_string(),
        )
            .into_response()
    }
}

async fn alerts(State(client): State<Client>) -> Result<Json<Vec<Alert>>, AppError> {
    let res = client
        .get("https://api.met.no/weatherapi/metalerts/1.1/.json")
        .send()
        .await
        .map_err(|_| AppError::new("request failed"))?;
    let json = res
        .json::<Value>()
        .await
        .map_err(|_| AppError::new("Deserialization failed"))?;
    let alerts: Vec<Alert> = json["features"]
        .as_array()
        .ok_or(AppError::new("Failed to convert value to alert type"))?
        .iter()
        .filter_map(|alert| MetAlert::try_from(alert.clone()).ok())
        .map(|alert| alert.into())
        .collect();
    Ok(Json(alerts))
}

async fn nowcasts(
    State(client): State<Client>,
    Query(location): Query<Location>,
) -> Result<Json<Vec<Nowcast>>, AppError> {
    let met_cast: MetNowcast = client
        .get("https://api.met.no/weatherapi/nowcast/2.0/complete")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .send()
        .await
        .map_err(|_| AppError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| AppError::new("Deserialization failed"))?
        .try_into()
        .map_err(|_| AppError::new("Failed to convert value to nowcast type"))?;
    let openweathermap: OpenWeatherNowcast = client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[("lat", location.lat), ("lon", location.lon)])
        .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
        .send()
        .await
        .map_err(|_| AppError::new("request failed"))?
        .json::<Value>()
        .await
        .map_err(|_| AppError::new("Deserialization failed"))?
        .try_into()
        .map_err(|_| AppError::new("Failed to convert value to nowcast type"))?;
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

    let app = Router::new()
        .route("/alerts", get(alerts))
        .route("/nowcasts", get(nowcasts))
        .with_state(client);

    Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
