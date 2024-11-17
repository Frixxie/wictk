mod alerts;
mod cache;
mod handlers;
mod locations;
mod nowcasts;

use axum::serve;
use handlers::Alerts;
use locations::OpenWeatherMapLocation;
use metrics_exporter_prometheus::PrometheusBuilder;
use nowcasts::Nowcast;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::cache::Cache;
use crate::handlers::setup_router;

#[derive(Clone, Debug)]
pub struct AppState {
    pub client: reqwest::Client,
    pub alert_cache: Cache<String, Alerts>,
    pub location_cache: Cache<String, Option<OpenWeatherMapLocation>>,
    pub nowcast_cache: Cache<String, Option<Nowcast>>,
}

impl AppState {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client,
            alert_cache: Cache::new(),
            location_cache: Cache::new(),
            nowcast_cache: Cache::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
    let metrics_handler = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install recorder/exporter");

    let client_builder = reqwest::Client::builder();
    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        " ",
        env!("CARGO_PKG_HOMEPAGE"),
    );
    let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();

    let app_state = AppState::new(client);

    let app = setup_router(app_state, metrics_handler);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;

    Ok(())
}
