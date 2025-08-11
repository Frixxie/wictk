use anyhow::Result;
use hem::{setup_device, setup_sensors, store_nowcast};
use reqwest::blocking::Client;
use structopt::StructOpt;
use wictk_core::{Lightning, Nowcast};

use crate::hem::store_lightning;

use rayon::prelude::*;

mod hem;

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(short, long, default_value = "Trondheim")]
    location: String,

    #[structopt(short, long, default_value = "http://wictk.frikk.io/")]
    service_url: String,

    #[structopt(short, long, default_value = "http://hemrs.frikk.io/")]
    hemrs_url: String,

    #[structopt(long)]
    store_lightning: bool,
}

pub fn get_nowcast(client: &Client, url: &str, location: &str) -> Result<Vec<Nowcast>> {
    let response = client
        .get(format!("{}api/nowcasts?location={}", url, location))
        .send()?;
    Ok(response.json()?)
}

pub fn get_lightnings(client: &Client, url: &str) -> Result<Vec<Lightning>> {
    let response = client.get(format!("{}api/recent_lightning", url)).send()?;
    Ok(response.json()?)
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let client = Client::new();
    let nowcasts = get_nowcast(&client, &opts.service_url, &opts.location)?;
    let sensors = setup_sensors(&client, &format!("{}api/sensors", &opts.hemrs_url))?;
    let device_met = setup_device(
        &client,
        &format!("{}api/devices", &opts.hemrs_url),
        "wictk_met",
        &opts.location,
    )?;
    let device_opm = setup_device(
        &client,
        &format!("{}api/devices", &opts.hemrs_url),
        "wictk_opm",
        &opts.location,
    )?;
    let device_lightning = setup_device(
        &client,
        &format!("{}api/devices", &opts.hemrs_url),
        "wictk_lightning",
        "Mobile",
    )?;
    println!("{}, {}", opts.location, nowcasts.first().unwrap());
    for nowcast in nowcasts {
        match nowcast.clone() {
            Nowcast::Met(_) => store_nowcast(
                &client,
                &format!("{}api/measurements", opts.hemrs_url),
                &nowcast,
                &device_met,
                &sensors,
            )?,
            Nowcast::OpenWeather(_) => store_nowcast(
                &client,
                &format!("{}api/measurements", opts.hemrs_url),
                &nowcast,
                &device_opm,
                &sensors,
            )?,
        }
    }
    if !opts.store_lightning {
        return Ok(());
    }
    let now = chrono::Utc::now();
    let lightnings = get_lightnings(&client, &opts.service_url)?;
    lightnings
        .par_iter()
        .filter(|lightning| {
            // Filter out lightnings that are too old (older than 10 minutes)
            let lightning_time = lightning.time.with_timezone(&chrono::Utc);
            now.signed_duration_since(lightning_time).num_minutes() < 10
        })
        .for_each(|lightning| {
            store_lightning(
                &client,
                &format!("{}api/measurements", opts.hemrs_url),
                &device_lightning,
                sensors.lon,
                sensors.lat,
                lightning,
            )
            .unwrap();
        });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn should_get_nowcast_successfully() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/api/nowcasts")
            .match_query(mockito::Matcher::UrlEncoded("location".into(), "Trondheim".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
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
            ]"#)
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
            },
            _ => panic!("Expected Met nowcast"),
        }
        
        mock.assert();
    }

    #[test]
    fn should_handle_empty_nowcast_response() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/api/nowcasts?location=TestLocation")
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
        let mock = server.mock("GET", "/api/nowcasts")
            .match_query(mockito::Matcher::UrlEncoded("location".into(), "ErrorLocation".into()))
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
        let mock = server.mock("GET", "/api/recent_lightning")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {
                    "time": "2025-08-11T12:00:00Z",
                    "location": {
                        "x": 10.0,
                        "y": 63.0
                    },
                    "magic_value": 42
                }
            ]"#)
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
        let mock = server.mock("GET", "/api/recent_lightning")
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
        let mock = server.mock("GET", "/api/recent_lightning")
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
        
        let opts = Opts::from_iter(&["client_logger"]);
        assert_eq!(opts.location, "Trondheim");
        assert_eq!(opts.service_url, "http://wictk.frikk.io/");
        assert_eq!(opts.hemrs_url, "http://hemrs.frikk.io/");
        assert_eq!(opts.store_lightning, false);
    }

    #[test]
    fn should_parse_opts_with_custom_values() {
        use structopt::StructOpt;
        
        let opts = Opts::from_iter(&[
            "client_logger",
            "--location", "Oslo",
            "--service-url", "http://custom.service.url/",
            "--hemrs-url", "http://custom.hemrs.url/",
            "--store-lightning"
        ]);
        
        assert_eq!(opts.location, "Oslo");
        assert_eq!(opts.service_url, "http://custom.service.url/");
        assert_eq!(opts.hemrs_url, "http://custom.hemrs.url/");
        assert_eq!(opts.store_lightning, true);
    }

    #[test]
    fn should_get_nowcast_with_multiple_types() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/api/nowcasts")
            .match_query(mockito::Matcher::UrlEncoded("location".into(), "Mixed".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
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
            ]"#)
            .create();

        let client = Client::new();
        let result = get_nowcast(&client, &format!("{}/", server.url()), "Mixed");
        
        assert!(result.is_ok());
        let nowcasts = result.unwrap();
        assert_eq!(nowcasts.len(), 2);
        
        // Verify we have both types
        let has_met = nowcasts.iter().any(|n| matches!(n, Nowcast::Met(_)));
        let has_open_weather = nowcasts.iter().any(|n| matches!(n, Nowcast::OpenWeather(_)));
        assert!(has_met);
        assert!(has_open_weather);
        
        mock.assert();
    }

    #[test]
    fn should_get_lightnings_with_multiple_entries() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/api/recent_lightning")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
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
            ]"#)
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
