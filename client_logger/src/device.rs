use anyhow::Result;
use serde::{Deserialize, Serialize};

pub type DeviceId = i32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Device {
    #[serde(skip_serializing)]
    pub id: i32,
    pub name: String,
    pub location: String,
}

pub fn fetch_devices(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<Device>> {
    let devices = client.get(url).send()?.json::<Vec<Device>>()?;
    Ok(devices)
}

pub fn setup_device(
    client: &reqwest::blocking::Client,
    url: &str,
    device_name: &str,
    device_location: &str,
) -> Result<DeviceId> {
    let devices = fetch_devices(client, url)?;
    let device = devices
        .iter()
        .find(|d| d.name == device_name && d.location == device_location);
    match device {
        Some(d) => {
            tracing::info!("Found existing device: {:?}", d);
            Ok(d.id)
        }
        None => {
            let new_device = Device {
                id: 0,
                name: device_name.to_string(),
                location: device_location.to_string(),
            };
            let response = client.post(url).json(&new_device).send()?;
            tracing::info!("Created new device: {:?}", response);
            setup_device(client, url, device_name, device_location)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn should_fetch_devices_successfully() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {"id": 1, "name": "test_device", "location": "test_location"},
                {"id": 2, "name": "another_device", "location": "another_location"}
            ]"#,
            )
            .create();

        let client = reqwest::blocking::Client::new();
        let result = fetch_devices(&client, &server.url());

        assert!(result.is_ok());
        let devices = result.unwrap();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, 1);
        assert_eq!(devices[0].name, "test_device");
        assert_eq!(devices[0].location, "test_location");

        mock.assert();
    }

    #[test]
    fn should_handle_fetch_devices_error() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/").with_status(500).create();

        let client = reqwest::blocking::Client::new();
        let result = fetch_devices(&client, &server.url());

        assert!(result.is_err());
        mock.assert();
    }

    #[test]
    fn should_setup_existing_device() {
        let mut server = Server::new();
        let mock_get = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {"id": 1, "name": "existing_device", "location": "test_location"}
            ]"#,
            )
            .create();

        let client = reqwest::blocking::Client::new();
        let result = setup_device(&client, &server.url(), "existing_device", "test_location");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        mock_get.assert();
    }

    #[test]
    fn should_setup_new_device() {
        let mut server = Server::new();

        let mock_get1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create();

        let mock_post = server
            .mock("POST", "/")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 2, "name": "new_device", "location": "new_location"}"#)
            .create();

        let mock_get2 = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {"id": 2, "name": "new_device", "location": "new_location"}
            ]"#,
            )
            .create();

        let client = reqwest::blocking::Client::new();
        let result = setup_device(&client, &server.url(), "new_device", "new_location");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        mock_get1.assert();
        mock_post.assert();
        mock_get2.assert();
    }

    #[test]
    fn should_compare_devices_for_equality() {
        let device1 = Device {
            id: 1,
            name: "test".to_string(),
            location: "location".to_string(),
        };

        let device2 = Device {
            id: 1,
            name: "test".to_string(),
            location: "location".to_string(),
        };

        assert_eq!(device1, device2);
    }

    #[test]
    fn should_serialize_device_correctly() {
        let device = Device {
            id: 1,
            name: "test_device".to_string(),
            location: "test_location".to_string(),
        };

        let json = serde_json::to_string(&device).unwrap();
        let expected = r#"{"name":"test_device","location":"test_location"}"#;
        assert_eq!(json, expected);
    }
}
