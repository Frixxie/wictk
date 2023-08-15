use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router, Server,
};
use geocoding::{Forward, Openstreetmap};
use hyper::StatusCode;
use reqwest::Client;
use serde_json::Value;
use weather_alert::{Alert, MetAlert};
use weather_nowcast::{MetNowcast, Nowcast};

mod weather_alert;
mod weather_nowcast;

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
        .filter_map(|alert| MetAlert::try_from(alert.clone()).ok())
        .map(|alert| alert.into())
        .collect();
    Ok(Json(alerts))
}

async fn nowcasts(
    State(client): State<Client>,
    Query(location): Query<String>,
) -> Result<Json<Nowcast>, StatusCode> {
    let osm = Openstreetmap::new();
    let lonlat: Vec<geo::Point<f32>> = osm
        .forward(&location)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let res = client
        .get("https://api.met.no/weatherapi/nowcast/2.0/complete")
        .query(&[
            ("lat", lonlat[0].x().to_string()),
            ("lon", lonlat[0].y().to_string()),
        ])
        .send()
        .await;
    let nowcast: MetNowcast = res
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<Value>()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(nowcast.into()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let app = Router::new()
        .route("/alerts", get(alerts))
        .route("/nowcasts", get(nowcasts))
        .with_state(client);

    Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
