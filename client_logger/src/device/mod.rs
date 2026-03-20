use anyhow::Result;

pub mod types;
pub mod device_client;
mod device_error;

pub use types::Device;
pub use device_client::DeviceClient;

pub type DeviceId = i32;

pub trait DeviceApi {
    async fn get_devices(&self, url: &str) -> Result<Vec<Device>>;
    async fn setup_device(
        &self,
        url: &str,
        device_name: &str,
        device_location: &str,
    ) -> Result<DeviceId>;
}
