mod handlers;
mod utils;
mod location;
mod alerts;
mod nowcasts;

use axum::{routing::get, Router, Server};
use simple_logger::SimpleLogger;

use crate::handlers::{alerts, get_geocoding, nowcasts};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .route("/geocoding", get(get_geocoding))
        .with_state(client);

    let status = Router::new().route("/ping", get(handlers::ping));

    let app = Router::new()
        .nest("/status", status)
        .nest("/api", api);

    Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
