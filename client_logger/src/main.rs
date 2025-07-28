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

fn get_nowcast(client: &Client, url: &str, location: &str) -> Result<Vec<Nowcast>> {
    let response = client
        .get(format!("{}api/nowcasts?location={}", url, location))
        .send()?;
    Ok(response.json()?)
}

fn get_lightnings(client: &Client, url: &str) -> Result<Vec<Lightning>> {
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
