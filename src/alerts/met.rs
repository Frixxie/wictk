use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::location::Coordinates;

use super::{
    alerts::{AlertError, AlertFetcher, Severity},
    Alert,
};

impl From<MetAlert> for Alert {
    fn from(met: MetAlert) -> Self {
        Alert::Met(met)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeDuration {
    from: DateTime<Utc>,
    until: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetAlert {
    pub title: String,
    pub severity: Severity,
    pub description: String,
    pub certainty: String,
    pub event: String,
    pub duration: TimeDuration,
}

impl TryFrom<serde_json::Value> for MetAlert {
    type Error = AlertError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let severity = match value["properties"]["severity"].as_str() {
            Some("Moderate") => Severity::Yellow,
            Some("Severe") => Severity::Orange,
            Some("Extreme") => Severity::Red,
            _ => {
                return Err(AlertError::new("invalid severity"));
            }
        };
        let title = value["properties"]["title"]
            .as_str()
            .ok_or_else(|| AlertError::new("Failed to parse title"))?
            .to_owned();
        let description = value["properties"]["description"]
            .as_str()
            .ok_or_else(|| AlertError::new("Failed to parse description"))?
            .to_owned();
        let certainty = value["properties"]["certainty"]
            .as_str()
            .ok_or_else(|| AlertError::new("Failed to parse certainty"))?
            .to_owned();
        let event = value["properties"]["event"]
            .as_str()
            .ok_or_else(|| AlertError::new("Failed to parse event"))?
            .to_owned();
        let duration = TimeDuration {
            from: value["when"]["interval"][0]
                .as_str()
                .ok_or_else(|| AlertError::new("Failed to parse from"))?
                .parse()
                .map_err(|_| AlertError::new("Failed to parse from"))?,
            until: value["when"]["interval"][1]
                .as_str()
                .ok_or_else(|| AlertError::new("Failed to parse until"))?
                .parse()
                .map_err(|_| AlertError::new("Failed to parse until"))?,
        };
        Ok(MetAlert {
            severity,
            title,
            description,
            certainty,
            event,
            duration,
        })
    }
}

#[async_trait]
impl AlertFetcher for MetAlert {
    async fn fetch(client: Client, _location: Coordinates) -> Result<Vec<Alert>, AlertError> {
        let result: Vec<Alert> = client
            .get("https://api.met.no/weatherapi/metalerts/1.1/.json")
            .send()
            .await
            .map_err(|err| {
                log::error!("Error {}", err);
                AlertError::new("Request to Met.no failed")
            })?
            .json::<Value>()
            .await
            .map_err(|err| {
                log::error!("Error {}", err);
                AlertError::new("Deserialization from Met.no failed")
            })?
            .get("features")
            .ok_or(AlertError::new(
                "Failed to convert get features value to alert type",
            ))?
            .as_array()
            .ok_or(AlertError::new("Failed to convert value to alert type"))?
            .iter()
            .filter_map(|alert| MetAlert::try_from(alert.clone()).ok())
            .map(|alert| alert.into())
            .collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn try_from_json() {
        let json = r#"{"geometry":{"coordinates":[[[15.4258,68.3229],[15.7923,68.2672],[14.5503,67.799],[13.9242,67.1765],[15.8189,67.0223],[16.0827,67.4949],[16.5759,67.5501],[16.7813,67.8797],[17.3572,68.0586],[17.9593,67.966],[18.0783,68.1858],[16.4639,68.4005],[16.8437,69.4366],[15.607,69.2479],[14.216,68.6853],[12.5634,67.8946],[12.9458,67.7361],[14.8027,68.2064],[15.4258,68.3229],[15.4258,68.3229],[15.4258,68.3229]]],"type":"Polygon"},"properties":{"area":"Lofoten, Vesterålen, og deler av Salten, Ofoten og Sør-Troms","awarenessResponse":"Følg med","awarenessSeriousness":"Utfordrende situasjon","awareness_level":"2; yellow; Moderate","awareness_type":"8; forest-fire","certainty":"Likely","consequences":"Vegetasjon kan lett antennes og store områder kan bli berørt. ","county":["18"],"description":"Update: Lokal skog- og lyngbrannfare inntil det kommer nedbør av betydning.","event":"forestFire","eventAwarenessName":"Skogbrannfare","geographicDomain":"land","id":"2.49.0.1.578.0.20230811073606.016","instruction":"Vær forsiktig med åpen ild. Følg lokale myndigheters instruksjoner. Behov for forebyggende tiltak og beredskap skal vurderes fortløpende av beredskapsaktører. ","resources":[{"description":"CAP file","mimeType":"application\/xml","uri":"https:\/\/api.met.no\/weatherapi\/metalerts\/1.1\/?cap=2.49.0.1.578.0.20230811073606.016"},{"description":"","mimeType":"image\/png","uri":"https:\/\/slaps.met.no\/cap-images\/abbba23b-e4df-44e9-896c-b9552e957166.png"}],"severity":"Moderate","title":"Skogbrannfare, gult nivå, Lofoten, Vesterålen, og deler av Salten, Ofoten og Sør-Troms, 2023-08-10T22:00:00+00:00, 2023-08-14T22:00:00+00:00","type":"Update"},"type":"Feature","when":{"interval":["2023-08-10T22:00:00+00:00","2023-08-14T22:00:00+00:00"]}}"#;

        let json_value: Value = serde_json::from_str(json).unwrap();

        let alert = MetAlert::try_from(json_value).unwrap();

        assert_eq!(alert.severity, Severity::Yellow);
        assert_eq!(
            alert.title,
            "Skogbrannfare, gult nivå, Lofoten, Vesterålen, og deler av Salten, Ofoten og Sør-Troms, 2023-08-10T22:00:00+00:00, 2023-08-14T22:00:00+00:00"
        );
        assert_eq!(
            alert.description,
            "Update: Lokal skog- og lyngbrannfare inntil det kommer nedbør av betydning."
        );
        assert_eq!(alert.certainty, "Likely");
        assert_eq!(alert.event, "forestFire");
        assert_eq!(
            alert.duration.from.to_rfc3339(),
            "2023-08-10T22:00:00+00:00"
        );
        assert_eq!(
            alert.duration.until.to_rfc3339(),
            "2023-08-14T22:00:00+00:00"
        );

    }

    #[tokio::test]
    async fn met_fetch() {
        let client = Client::new();
        let location = Coordinates::new(68.3229, 15.4258);
        let alerts = MetAlert::fetch(client, location).await;
        assert!(alerts.is_ok());
    }
}
