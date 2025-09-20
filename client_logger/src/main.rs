use anyhow::Result;
use device::setup_device;
use reqwest::blocking::Client;
use sensor::setup_sensors;
use storage::{store_lightning, store_met_nowcast, store_openweather_nowcast};
use structopt::StructOpt;
use tracing::{instrument, Level};
use tracing_subscriber::FmtSubscriber;
use wictk_core::{Lightning, Nowcast};

use rayon::prelude::*;

use crate::measurement::{calculate_temperature_ratio, fetch_measurements};

mod device;
mod measurement;
mod sensor;
mod storage;

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

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(short, long, required = true)]
    locations: Vec<String>,

    #[structopt(short, long, default_value = "http://wictk.frikk.io/")]
    service_url: String,

    #[structopt(short = "r", long, default_value = "http://hemrs.frikk.io/")]
    hemrs_url: String,

    #[structopt(long)]
    store_lightning: bool,

    #[structopt(long, default_value = "info")]
    log_level: LogLevel,
}

#[instrument(skip(client), fields(url = %url, location = %location))]
pub fn get_nowcast(client: &Client, url: &str, location: &str) -> Result<Vec<Nowcast>> {
    tracing::debug!("Fetching nowcast data");
    let full_url = format!("{url}api/nowcasts?location={location}");
    tracing::info!("Requesting nowcast data from: {}", full_url);

    let response = client.get(&full_url).send()?;

    tracing::debug!("Response status: {}", response.status());

    if response.status().is_success() {
        let nowcasts: Vec<Nowcast> = response.json()?;
        tracing::info!("Successfully fetched {} nowcast records", nowcasts.len());
        Ok(nowcasts)
    } else {
        tracing::error!("Failed to fetch nowcast data: HTTP {}", response.status());
        Err(anyhow::anyhow!("HTTP error: {}", response.status()))
    }
}

#[instrument(skip(client), fields(url = %url))]
pub fn get_lightnings(client: &Client, url: &str) -> Result<Vec<Lightning>> {
    tracing::debug!("Fetching lightning data");
    let full_url = format!("{url}api/recent_lightning");
    tracing::info!("Requesting lightning data from: {}", full_url);

    let response = client.get(&full_url).send()?;

    tracing::debug!("Response status: {}", response.status());

    if response.status().is_success() {
        let lightnings: Vec<Lightning> = response.json()?;
        tracing::info!(
            "Successfully fetched {} lightning records",
            lightnings.len()
        );
        Ok(lightnings)
    } else {
        tracing::error!("Failed to fetch lightning data: HTTP {}", response.status());
        Err(anyhow::anyhow!("HTTP error: {}", response.status()))
    }
}

fn main() -> Result<()> {
    let opts = Opts::from_args();

    let level: Level = opts.log_level.clone().into();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    tracing::info!("Starting client logger with configuration: {:?}", opts);

    let client = Client::new();
    tracing::info!("HTTP client initialized successfully");

    // Setup sensors (global, not per location)
    let setup_start = std::time::Instant::now();
    let sensors = match setup_sensors(&client, &format!("{}api/sensors", &opts.hemrs_url)) {
        Ok(sensors) => {
            tracing::info!("Sensors setup completed successfully");
            sensors
        }
        Err(e) => {
            tracing::error!("Failed to setup sensors: {}", e);
            return Err(e);
        }
    };

    // Setup lightning device (global)
    let device_lightning = match setup_device(
        &client,
        &format!("{}api/devices", &opts.hemrs_url),
        "wictk_lightning",
        "Mobile",
    ) {
        Ok(device_id) => {
            tracing::info!("Lightning device setup completed (ID: {})", device_id);
            device_id
        }
        Err(e) => {
            tracing::error!("Failed to setup lightning device: {}", e);
            return Err(e);
        }
    };

    let setup_elapsed = setup_start.elapsed();
    tracing::info!(
        "Global setup completed in {:.2}s",
        setup_elapsed.as_secs_f64()
    );

    // Process each location
    for location in &opts.locations {
        tracing::info!("Processing location: {}", location);

        // Fetch nowcast data for this location
        let start_time = std::time::Instant::now();
        let nowcasts = match get_nowcast(&client, &opts.service_url, location) {
            Ok(data) => {
                let elapsed = start_time.elapsed();
                tracing::info!(
                    "Successfully retrieved nowcast data for {} in {:.2}s",
                    location,
                    elapsed.as_secs_f64()
                );
                data
            }
            Err(e) => {
                tracing::error!("Failed to fetch nowcast data for {}: {}", location, e);
                continue; // Continue with other locations
            }
        };

        if nowcasts.is_empty() {
            tracing::warn!("No nowcast data available for location: {}", location);
            continue;
        }

        // Setup devices for this location
        let device_met = match setup_device(
            &client,
            &format!("{}api/devices", &opts.hemrs_url),
            "wictk_met",
            location,
        ) {
            Ok(device_id) => {
                tracing::info!("MET device setup completed for {} (ID: {})", location, device_id);
                device_id
            }
            Err(e) => {
                tracing::error!("Failed to setup MET device for {}: {}", location, e);
                continue;
            }
        };

        let device_opm = match setup_device(
            &client,
            &format!("{}api/devices", &opts.hemrs_url),
            "wictk_opm",
            location,
        ) {
            Ok(device_id) => {
                tracing::info!("OpenWeatherMap device setup completed for {} (ID: {})", location, device_id);
                device_id
            }
            Err(e) => {
                tracing::error!("Failed to setup OpenWeatherMap device for {}: {}", location, e);
                continue;
            }
        };

        // Log the first nowcast for debugging
        if let Some(first_nowcast) = nowcasts.first() {
            tracing::info!("First nowcast for {}: {}", location, first_nowcast);
        }

        // Store nowcast data
        let temp = match fetch_measurements(&client, &opts.hemrs_url, 10, 9) {
            Ok(temp) => temp,
            Err(e) => {
                tracing::error!("Failed to fetch measurements for {}: {}", location, e);
                continue;
            }
        };
        let storage_start = std::time::Instant::now();
        let mut met_count = 0;
        let mut opm_count = 0;

        for (index, nowcast) in nowcasts.iter().enumerate() {
            tracing::debug!("Processing nowcast {} of {} for {}", index + 1, nowcasts.len(), location);

            let result = match nowcast.clone() {
                Nowcast::Met(met_nowcast) => {
                    met_count += 1;
                    let ratio = calculate_temperature_ratio(
                        met_nowcast.air_temperature,
                        temp.value
                    );
                    tracing::debug!("Temperature ratio for {}: {:?}", location, ratio);

                    tracing::debug!("Storing MET nowcast data for {}", location);
                    store_met_nowcast(
                        &client,
                        &format!("{}api/measurements", opts.hemrs_url),
                        &met_nowcast,
                        &device_met,
                        &sensors,
                    )
                }
                Nowcast::OpenWeather(openweather_nowcast) => {
                    opm_count += 1;
                    tracing::debug!("Storing OpenWeatherMap nowcast data for {}", location);
                    store_openweather_nowcast(
                        &client,
                        &format!("{}api/measurements", opts.hemrs_url),
                        &openweather_nowcast,
                        &device_opm,
                        &sensors,
                    )
                }
            };

            match result {
                Ok(()) => tracing::debug!("Successfully stored nowcast {} for {}", index + 1, location),
                Err(e) => {
                    tracing::error!("Failed to store nowcast {} for {}: {}", index + 1, location, e);
                    continue;
                }
            }
        }

        let storage_elapsed = storage_start.elapsed();
        tracing::info!(
            "Stored {} nowcasts for {} ({} MET, {} OpenWeatherMap) in {:.2}s",
            nowcasts.len(),
            location,
            met_count,
            opm_count,
            storage_elapsed.as_secs_f64()
        );
    }



    // Handle lightning data if requested
    if !opts.store_lightning {
        tracing::info!("Lightning storage disabled - skipping lightning data");
        return Ok(());
    }

    let now = chrono::Utc::now();
    tracing::info!("Current time for lightning filtering: {}", now);

    let lightnings = match get_lightnings(&client, &opts.service_url) {
        Ok(data) => {
            tracing::info!("Successfully retrieved lightning data");
            data
        }
        Err(e) => {
            tracing::error!("Failed to fetch lightning data: {}", e);
            return Err(e);
        }
    };

    if lightnings.is_empty() {
        tracing::info!("No lightning data available");
        return Ok(());
    }

    let recent_lightnings: Vec<&Lightning> = lightnings
        .par_iter()
        .filter(|lightning| {
            // Filter out lightnings that are too old (older than 10 minutes)
            let lightning_time = lightning.time.with_timezone(&chrono::Utc);
            let age_minutes = now.signed_duration_since(lightning_time).num_minutes();
            let is_recent = age_minutes < 10;

            if !is_recent {
                tracing::debug!("Filtering out old lightning: {} minutes old", age_minutes);
            }

            is_recent
        })
        .collect();

    tracing::info!(
        "Filtered {} recent lightnings from {} total (within 10 minutes)",
        recent_lightnings.len(),
        lightnings.len()
    );

    if recent_lightnings.is_empty() {
        tracing::info!("No recent lightning data to store");
        return Ok(());
    }

    tracing::info!(
        "Storing {} recent lightning records",
        recent_lightnings.len()
    );

    // Store lightning data in parallel using rayon
    recent_lightnings
        .par_iter()
        .enumerate()
        .for_each(|(index, lightning)| {
            tracing::debug!(
                "Storing lightning {} of {}, time: {}",
                index + 1,
                recent_lightnings.len(),
                lightning.time
            );
            match store_lightning(
                &client,
                &format!("{}api/measurements", opts.hemrs_url),
                &device_lightning,
                sensors.lon,
                sensors.lat,
                lightning,
            ) {
                Ok(()) => {
                    tracing::debug!("Successfully stored lightning {}", index + 1);
                }
                Err(e) => {
                    tracing::error!("Failed to store lightning {}: {}", index + 1, e);
                }
            }
        });

    tracing::info!("Stored {} lightning records", recent_lightnings.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn should_get_nowcast_successfully() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/nowcasts")
            .match_query(mockito::Matcher::UrlEncoded(
                "location".into(),
                "Trondheim".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "met": {
                        "time": "2025-08-11T12:00:00Z",
                        "location": {"lon": 10.0, "lat": 63.0},
                        "description": "Clear",
                        "air_temperature": 20.5,
                        "relative_humidity": 65.0,
                        "precipitation_rate": 0.0,
                        "precipitation_amount": 0.0,
                        "wind_speed": 5.2,
                        "wind_speed_gust": 6.0,
                        "wind_from_direction": 180.0
                    }
                }
            ]"#,
            )
            .create();

        let client = Client::new();
        let result = get_nowcast(&client, &format!("{}/", server.url()), "Trondheim");

        assert!(result.is_ok());
        let nowcasts = result.unwrap();
        assert_eq!(nowcasts.len(), 1);

        match &nowcasts[0] {
            Nowcast::Met(met) => {
                assert_eq!(met.air_temperature, 20.5);
                assert_eq!(met.relative_humidity, 65.0);
                assert_eq!(met.wind_speed, 5.2);
                assert_eq!(met.wind_from_direction, 180.0);
            }
            _ => panic!("Expected Met nowcast"),
        }

        mock.assert();
    }

    #[test]
    fn should_handle_empty_nowcast_response() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/nowcasts")
            .match_query(mockito::Matcher::UrlEncoded(
                "location".into(),
                "TestLocation".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create();

        let client = Client::new();
        let result = get_nowcast(&client, &format!("{}/", server.url()), "TestLocation");

        assert!(result.is_ok());
        let nowcasts = result.unwrap();
        assert_eq!(nowcasts.len(), 0);

        mock.assert();
    }

    #[test]
    fn should_handle_nowcast_server_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/nowcasts")
            .match_query(mockito::Matcher::UrlEncoded(
                "location".into(),
                "ErrorLocation".into(),
            ))
            .with_status(500)
            .create();

        let client = Client::new();
        let result = get_nowcast(&client, &format!("{}/", server.url()), "ErrorLocation");

        assert!(result.is_err());
        mock.assert();
    }

    #[test]
    fn should_get_lightnings_successfully() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/recent_lightning")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "time": "2025-08-11T12:00:00Z",
                    "location": {
                        "x": 10.0,
                        "y": 63.0
                    },
                    "magic_value": 42
                }
            ]"#,
            )
            .create();

        let client = Client::new();
        let result = get_lightnings(&client, &format!("{}/", server.url()));

        assert!(result.is_ok());
        let lightnings = result.unwrap();
        assert_eq!(lightnings.len(), 1);

        let lightning = &lightnings[0];
        assert_eq!(lightning.location.x(), 10.0);
        assert_eq!(lightning.location.y(), 63.0);
        assert_eq!(lightning.magic_value, 42);

        mock.assert();
    }

    #[test]
    fn should_handle_empty_lightnings_response() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/recent_lightning")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create();

        let client = Client::new();
        let result = get_lightnings(&client, &format!("{}/", server.url()));

        assert!(result.is_ok());
        let lightnings = result.unwrap();
        assert_eq!(lightnings.len(), 0);

        mock.assert();
    }

    #[test]
    fn should_handle_lightnings_server_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/recent_lightning")
            .with_status(500)
            .create();

        let client = Client::new();
        let result = get_lightnings(&client, &format!("{}/", server.url()));

        assert!(result.is_err());
        mock.assert();
    }

    #[test]
    fn should_parse_opts_with_default_values() {
        use structopt::StructOpt;

        let opts = Opts::from_iter(&["client_logger", "--locations", "Trondheim"]);
        assert_eq!(opts.locations, vec!["Trondheim"]);
        assert_eq!(opts.service_url, "http://wictk.frikk.io/");
        assert_eq!(opts.hemrs_url, "http://hemrs.frikk.io/");
        assert!(!opts.store_lightning);
    }

    #[test]
    fn should_parse_opts_with_custom_values() {
        use structopt::StructOpt;

        let opts = Opts::from_iter(&[
            "client_logger",
            "--locations",
            "Oslo",
            "--locations",
            "Bergen",
            "--service-url",
            "http://custom.service.url/",
            "--hemrs-url",
            "http://custom.hemrs.url/",
            "--store-lightning",
        ]);

        assert_eq!(opts.locations, vec!["Oslo", "Bergen"]);
        assert_eq!(opts.service_url, "http://custom.service.url/");
        assert_eq!(opts.hemrs_url, "http://custom.hemrs.url/");
        assert!(opts.store_lightning);
    }

    #[test]
    fn should_get_nowcast_with_multiple_types() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/nowcasts")
            .match_query(mockito::Matcher::UrlEncoded(
                "location".into(),
                "Mixed".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "met": {
                        "time": "2025-08-11T12:00:00Z",
                        "location": {"lon": 10.0, "lat": 63.0},
                        "description": "Clear",
                        "air_temperature": 20.5,
                        "relative_humidity": 65.0,
                        "precipitation_rate": 0.0,
                        "precipitation_amount": 0.0,
                        "wind_speed": 5.2,
                        "wind_speed_gust": 6.0,
                        "wind_from_direction": 180.0
                    }
                },
                {
                    "open_weather": {
                        "dt": "2025-08-11T13:00:00Z",
                        "name": "Mixed",
                        "country": "NO",
                        "lon": 10.0,
                        "lat": 63.0,
                        "main": "Clouds",
                        "desc": "few clouds",
                        "clouds": 20,
                        "wind_speed": 4.1,
                        "wind_deg": 200,
                        "visibility": 10000,
                        "temp": 22.3,
                        "feels_like": 23.0,
                        "humidity": 70,
                        "pressure": 1013
                    }
                }
            ]"#,
            )
            .create();

        let client = Client::new();
        let result = get_nowcast(&client, &format!("{}/", server.url()), "Mixed");

        assert!(result.is_ok());
        let nowcasts = result.unwrap();
        assert_eq!(nowcasts.len(), 2);

        // Verify we have both types
        let has_met = nowcasts.iter().any(|n| matches!(n, Nowcast::Met(_)));
        let has_open_weather = nowcasts
            .iter()
            .any(|n| matches!(n, Nowcast::OpenWeather(_)));
        assert!(has_met);
        assert!(has_open_weather);

        mock.assert();
    }

    #[test]
    fn should_get_lightnings_with_multiple_entries() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/api/recent_lightning")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "time": "2025-08-11T12:00:00Z",
                    "location": {
                        "x": 10.0,
                        "y": 63.0
                    },
                    "magic_value": 42
                },
                {
                    "time": "2025-08-11T12:05:00Z",
                    "location": {
                        "x": 11.0,
                        "y": 64.0
                    },
                    "magic_value": 24
                }
            ]"#,
            )
            .create();

        let client = Client::new();
        let result = get_lightnings(&client, &format!("{}/", server.url()));

        assert!(result.is_ok());
        let lightnings = result.unwrap();
        assert_eq!(lightnings.len(), 2);

        // Verify first lightning
        assert_eq!(lightnings[0].location.x(), 10.0);
        assert_eq!(lightnings[0].location.y(), 63.0);
        assert_eq!(lightnings[0].magic_value, 42);

        // Verify second lightning
        assert_eq!(lightnings[1].location.x(), 11.0);
        assert_eq!(lightnings[1].location.y(), 64.0);
        assert_eq!(lightnings[1].magic_value, 24);

        mock.assert();
    }
}
