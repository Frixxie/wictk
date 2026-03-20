use anyhow::Result;
use clap::Parser;
use device::{DeviceApi, DeviceClient};
use sensor::{SensorApi, SensorClient, SensorIds};
use storage::{StorageApi, StorageClient};
use weather::{WeatherApi, WeatherClient};
use wictk_core::{Lightning, Nowcast};

mod cli;
mod device;
mod measurement;
mod sensor;
mod storage;
mod weather;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = cli::Opts::parse();
    cli::init_tracing(&opts);

    tracing::info!("Starting client logger with configuration: {:?}", opts);

    let client = reqwest::Client::new();
    tracing::info!("HTTP client initialized successfully");

    let device_client = DeviceClient::new(client.clone());
    let sensor_client = SensorClient::new(client.clone());
    let storage_client = StorageClient::new(client.clone());
    let weather_client = WeatherClient::new(client.clone());

    // Setup sensors (global, not per location)
    let setup_start = std::time::Instant::now();
    let sensor_url = format!("{}api/sensors", opts.service_url);
    let sensors: SensorIds = sensor_client.setup_sensors(&sensor_url).await?;

    // Setup lightning device (global)
    let device_url = format!("{}api/devices", opts.service_url);
    let device_lightning: device::DeviceId = device_client
        .setup_device(&device_url, "lightning", "global")
        .await?;

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
        let nowcasts = match weather_client
            .get_nowcast(&opts.service_url, location)
            .await
        {
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
        let device_met: device::DeviceId = device_client
            .setup_device(&device_url, "met", location)
            .await?;

        let device_opm: device::DeviceId = device_client
            .setup_device(&device_url, "openweathermap", location)
            .await?;

        // Log the first nowcast for debugging
        if let Some(first_nowcast) = nowcasts.first() {
            tracing::info!("First nowcast for {}: {}", location, first_nowcast);
        }

        let storage_url = format!("{}api/measurements", opts.service_url);
        let storage_start = std::time::Instant::now();
        let mut met_count = 0;
        let mut opm_count = 0;

        for (index, nowcast) in nowcasts.iter().enumerate() {
            tracing::debug!(
                "Processing nowcast {} of {} for {}",
                index + 1,
                nowcasts.len(),
                location
            );

            let result: Result<()> = match nowcast.clone() {
                Nowcast::Met(met_nowcast) => {
                    met_count += 1;
                    tracing::debug!("Storing MET nowcast data for {}", location);
                    storage_client
                        .store_met_nowcast(&storage_url, &met_nowcast, &device_met, &sensors)
                        .await
                }
                Nowcast::OpenWeather(openweather_nowcast) => {
                    opm_count += 1;
                    tracing::debug!("Storing OpenWeatherMap nowcast data for {}", location);
                    storage_client
                        .store_openweather_nowcast(
                            &storage_url,
                            &openweather_nowcast,
                            &device_opm,
                            &sensors,
                        )
                        .await
                }
            };

            match result {
                Ok(()) => {
                    tracing::debug!("Successfully stored nowcast {} for {}", index + 1, location)
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to store nowcast {} for {}: {}",
                        index + 1,
                        location,
                        e
                    );
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

    let lightnings = match weather_client.get_lightnings(&opts.service_url).await {
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
        .iter()
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

    let storage_url = format!("{}api/measurements", opts.service_url);
    let recent_lightnings: Vec<Lightning> = recent_lightnings.into_iter().cloned().collect();
    if let Err(e) = storage_client
        .store_lightnings(
            &storage_url,
            &device_lightning,
            sensors.lon,
            sensors.lat,
            &recent_lightnings,
        )
        .await
    {
        tracing::error!("Failed to store lightning batch: {}", e);
        return Err(e);
    }

    tracing::info!("Stored {} lightning records", recent_lightnings.len());

    Ok(())
}
