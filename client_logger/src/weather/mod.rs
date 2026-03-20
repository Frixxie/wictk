use anyhow::Result;

pub mod weather_client;
mod weather_error;

pub use weather_client::WeatherClient;

pub trait WeatherApi {
    async fn get_nowcast(
        &self,
        url: &str,
        location: &str,
    ) -> Result<Vec<wictk_core::Nowcast>>;

    async fn get_lightnings(&self, url: &str) -> Result<Vec<wictk_core::Lightning>>;
}
