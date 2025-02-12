use anyhow::Result;
use reqwest::blocking::Client;
use structopt::StructOpt;
use wictk_core::Nowcast;

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(short, long, default_value = "Trondheim")]
    location: String,
    #[structopt(short, long, default_value = "http://desktop:3010/api/met/nowcasts")]
    url: String,
}

fn get_nowcast(client: Client, url: &str) -> Result<Nowcast> {
    let response = client.get(url).send()?;
    let nowcasts: Nowcast = response.json()?;
    Ok(nowcasts)
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let client = Client::new();
    let url = format!("{}?location={}", opts.url, opts.location);
    let nowcast = get_nowcast(client, &url)?;
    println!("{},{}", opts.location, nowcast);
    Ok(())
}
