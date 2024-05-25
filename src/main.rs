mod alerts;
mod cache;
mod handlers;
mod locations;
mod nowcasts;

use axum::serve;
use handlers::Alerts;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use crate::cache::Cache;
use crate::handlers::setup_router;

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

    let app = setup_router(app_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;

    Ok(())
}
