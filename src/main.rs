mod alerts;
mod cache;
mod handlers;
mod location;
mod nowcasts;
mod utils;

use axum::{routing::get, serve, Router};
use handlers::ping;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use crate::{
    cache::Cache,
    handlers::{alerts, geocoding, nowcasts, Alerts},
};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub cache: Cache<String, Alerts>,
}

impl AppState {
    pub fn new(client: reqwest::Client, alert_cache: Cache<String, Alerts>) -> Self {
        Self {
            client,
            cache: alert_cache,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let client_builder = reqwest::Client::builder();
    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        " ",
        env!("CARGO_PKG_HOMEPAGE"),
    );
    let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
    let alert_cache: Cache<String, Alerts> = Cache::new();

    let app_state = AppState::new(client, alert_cache);

    let api = Router::new()
        .route("/alerts", get(alerts))
        .route("/nowcasts", get(nowcasts))
        .route("/geocoding", get(geocoding))
        .with_state(app_state);

    let status = Router::new().route("/ping", get(ping));

    let app = Router::new().nest("/status", status).nest("/api", api);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;

    Ok(())
}
