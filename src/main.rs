mod alerts;
mod handlers;
mod location;
mod nowcasts;
mod utils;

use axum::{routing::get, serve, Router};
use handlers::ping;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use crate::handlers::{alerts, geocoding, nowcasts};

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

    let api = Router::new()
        .route("/alerts", get(alerts))
        .route("/nowcasts", get(nowcasts))
        .route("/geocoding", get(geocoding))
        .with_state(client);

    let status = Router::new().route("/ping", get(ping));

    let app = Router::new().nest("/status", status).nest("/api", api);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;

    Ok(())
}
