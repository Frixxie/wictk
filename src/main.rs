use axum::{extract::State, routing::get, Router, Server};
use hyper::StatusCode;
use reqwest::Client;

async fn alerts(State(client): State<Client>) -> Result<String, StatusCode> {
    let res = client
        .get("https://api.met.no/weatherapi/metalerts/1.1/.json")
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    match res?.text().await {
        Ok(text) => Ok(text),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
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
