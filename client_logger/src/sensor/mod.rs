use anyhow::Result;

pub mod sensor;
pub mod sensor_client;
mod sensor_error;

pub use sensor::{Sensor, SensorIds};
pub use sensor_client::SensorClient;

pub trait SensorApi {
    async fn get_sensors(&self, url: &str) -> Result<Vec<Sensor>>;
    async fn setup_sensor(
        &self,
        url: &str,
        sensor_name: &str,
        sensor_unit: &str,
    ) -> Result<i32>;
    async fn setup_sensors(&self, url: &str) -> Result<SensorIds>;
}
