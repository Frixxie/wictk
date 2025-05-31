use anyhow::Result;
use hem::{setup_device, setup_sensors, store_nowcast};
use reqwest::blocking::Client;
use structopt::StructOpt;
use wictk_core::Nowcast;

mod hem;

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(short, long, default_value = "Trondheim")]
    location: String,

    #[structopt(short, long, default_value = "http://wictk.frikk.io/api/nowcasts")]
    service_url: String,

    #[structopt(short, long, default_value = "http://hemrs.frikk.io/")]
    hemrs_url: String,
}

fn get_nowcast(client: &Client, url: &str) -> Result<Vec<Nowcast>> {
    let response = client.get(url).send()?;
    Ok(response.json()?)
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let client = Client::new();
    let url = format!("{}?location={}", opts.service_url, opts.location);
    let nowcasts = get_nowcast(&client, &url)?;
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
    Ok(())
}
