use axum::{extract::State, routing::get, Json, Router, Server};
use hyper::StatusCode;
use reqwest::Client;
use serde_json::Value;
use weather_alert::Alert;

mod weather_alert;

async fn alerts(State(client): State<Client>) -> Result<Json<Vec<Alert>>, StatusCode> {
    let res = client
        .get("https://api.met.no/weatherapi/metalerts/1.1/.json")
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let json = res
        .json::<Value>()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let alerts: Vec<Alert> = json["features"]
        .as_array()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .iter()
        .filter_map(|alert| weather_alert::Met::try_from(alert.clone()).ok())
        .map(|alert| alert.into())
        .collect();
    Ok(Json(alerts))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let app = Router::new()
        .route("/alerts", get(alerts))
        .with_state(client);

    Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
