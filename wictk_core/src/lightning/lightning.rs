use anyhow::Result;
use chrono::{DateTime, Utc};
use geo::Point;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lightning {
    pub location: Point,
    pub time: DateTime<Utc>,
    pub magic_value: u8,
}

impl Lightning {
    pub fn new(location: Point, time: DateTime<Utc>, magic_value: u8) -> Self {
        Lightning {
            location,
            time,
            magic_value,
        }
    }

    pub async fn find_ligntning(client: &Client, url: &str) -> Result<Vec<Lightning>> {
        let response = client.get(url).send().await?.json::<Value>().await?;
        let response_string = response
            .get("historicalData")
            .ok_or_else(|| {
                anyhow::anyhow!("Invalid response format: 'historicalData' field is missing")
            })?
            .to_string();

        let data = response_string.trim_matches('"');

        let lightning_data: Value = serde_json::from_str(&data)
            .map_err(|e| anyhow::anyhow!("Failed to parse lightning data: {}", e))?;

        let lightning_data = lightning_data
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected an array of lightning events"))?
            .into_iter()
            .filter_map(|event| {
                // event is on the form: [timestamp, latitude, longitude, magic_value]
                match event.as_array() {
                    Some(arr) if arr.len() == 4 => {
                        let timestamp = arr[0].as_i64()?;
                        let latitude = arr[1].as_f64()?;
                        let longitude = arr[2].as_f64()?;
                        let magic_value = arr[3].as_u64()? as u8;

                        let time = DateTime::<Utc>::from_timestamp(timestamp, 0);
                        match time {
                            Some(time) => {
                                let location = Point::new(longitude, latitude);
                                Some(Lightning::new(location, time, magic_value))
                            }
                            None => None,
                        }
                    }
                    _ => None,
                }
            })
            .collect::<Vec<Lightning>>();
        Ok(lightning_data)
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use geo::point;

    #[test]
    fn test_lightning_creation() {
        let location = point!(x: 10.0, y: 20.0);
        let time = Utc::now();
        let magic_value = 42;

        let lightning = Lightning::new(location, time, magic_value);
        assert_eq!(lightning.location, location);
        assert_eq!(lightning.time, time);
        assert_eq!(lightning.magic_value, magic_value);
    }

    #[tokio::test]
    async fn test_find_lightning() -> Result<()> {
        let client = Client::new();
        let url = "https://www.yr.no/api/v0/lightning-events?fromHours=24";
        let lightning_data = Lightning::find_ligntning(&client, url).await?;

        assert!(!lightning_data.is_empty());
        Ok(())
    }
}
