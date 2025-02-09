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
use redact::Secret;
use structopt::StructOpt;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::cache::Cache;
use crate::handlers::setup_router;

#[derive(Debug, Clone)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err("unknown log level".to_string()),
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "0.0.0.0:3000")]
    host: String,

    #[structopt(short, long, env = "OPENWEATHERMAPAPIKEY")]
    apikey: String,

    #[structopt(short, long, default_value = "info")]
    log_level: LogLevel,
}

impl From<LogLevel> for Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub openweathermap_apikey: Secret<String>,
    pub client: reqwest::Client,
    pub alert_cache: Cache<String, Alerts>,
    pub location_cache: Cache<String, Option<OpenWeatherMapLocation>>,
    pub nowcast_cache: Cache<String, Option<Nowcast>>,
}

impl AppState {
    pub fn new(client: reqwest::Client, apikey: String) -> Self {
        Self {
            openweathermap_apikey: Secret::new(apikey),
            client,
            alert_cache: Cache::new(),
            location_cache: Cache::new(),
            nowcast_cache: Cache::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::from_args();

    let level: Level = opts.log_level.into();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
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

    let app_state = AppState::new(client, opts.apikey);

    let app = setup_router(app_state, metrics_handler);

    let listener = TcpListener::bind(opts.host).await?;
    serve(listener, app).await?;

    Ok(())
}
