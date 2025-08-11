use axum::{extract::{Query, State}, Json};
use geo::{Distance, Haversine, Point};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use wictk_core::Lightning;

use crate::AppState;

use super::{error::ApplicationError, nowcasts::{find_location, LocationQuery}};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LightningQuery {
    #[serde(flatten)]
    pub location: Option<LocationQuery>,
    pub radius_km: Option<f64>,
}

#[instrument]
pub async fn get_recent_lightning(
    app_state: State<AppState>,
    Query(query): Query<LightningQuery>,
) -> Result<Json<Vec<Lightning>>, ApplicationError> {
    // Get the lightning data first
    let lightning_data = match app_state.lightning_cache.get("recent_lightning").await {
        Some(lightning) => lightning,
        None => {
            let lightning = Lightning::find_ligntning(
                &app_state.client,
                "https://www.yr.no/api/v0/lightning-events?fromHours=24",
            )
            .await
            .map_err(|err| {
                error!("Error fetching lightning data: {:?}", err);
                ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
            app_state
                .lightning_cache
                .insert("recent_lightning".to_string(), lightning.clone())
                .await;
            lightning
        }
    };

    // If no location is provided, return all lightning data
    let Some(location_query) = query.location else {
        return Ok(Json(lightning_data));
    };

    // Find the location coordinates
    let location_coords = find_location(
        location_query,
        &app_state.client,
        &app_state.location_cache,
        &app_state.openweathermap_apikey,
    )
    .await
    .map_err(|err| {
        error!("Error finding location: {:?}", err);
        ApplicationError::new(&err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    // Convert to geo::Point for distance calculations
    let query_point = Point::new(location_coords.lon as f64, location_coords.lat as f64);
    
    // Default radius to 50km if not specified
    let radius_meters = query.radius_km.unwrap_or(50.0) * 1000.0;

    // Filter lightning strikes within the specified radius
    let filtered_lightning: Vec<Lightning> = lightning_data
        .into_iter()
        .filter(|lightning| {
            let lightning_point = Point::new(lightning.location.x(), lightning.location.y());
            let distance = Haversine.distance(query_point, lightning_point);
            distance <= radius_meters
        })
        .collect();

    Ok(Json(filtered_lightning))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lightning_query_city() {
        let json = r#"{"location": "Oslo", "radius_km": 100.0}"#;
        let query: LightningQuery = serde_json::from_str(json).unwrap();
        
        match query.location.unwrap() {
            LocationQuery::Location(city) => assert_eq!(city.location, "Oslo"),
            _ => panic!("Expected city location"),
        }
        assert_eq!(query.radius_km, Some(100.0));
    }

    #[test]
    fn test_lightning_query_coordinates() {
        let json = r#"{"lat": "63.4308", "lon": "10.4034", "radius_km": 50.0}"#;
        let query: LightningQuery = serde_json::from_str(json).unwrap();
        
        match query.location.unwrap() {
            LocationQuery::Coordinates(coords) => {
                assert_eq!(coords.lat, "63.4308");
                assert_eq!(coords.lon, "10.4034");
            },
            _ => panic!("Expected coordinate location"),
        }
        assert_eq!(query.radius_km, Some(50.0));
    }

    #[test]
    fn test_lightning_query_no_location() {
        let json = r#"{}"#;
        let query: LightningQuery = serde_json::from_str(json).unwrap();
        
        assert!(query.location.is_none());
        assert_eq!(query.radius_km, None);
    }

    #[test]
    fn test_lightning_query_default_radius() {
        let json = r#"{"location": "Trondheim"}"#;
        let query: LightningQuery = serde_json::from_str(json).unwrap();
        
        assert!(query.location.is_some());
        assert_eq!(query.radius_km, None);
    }

    #[test]
    fn test_distance_calculation() {
        // Create two points: Oslo and Trondheim (approximately 388km apart)
        let oslo = Point::new(10.7522, 59.9139);
        let trondheim = Point::new(10.4034, 63.4308);
        
        let distance = Haversine.distance(oslo, trondheim);
        
        // Distance should be approximately 388km (388000 meters)
        assert!((distance - 388000.0_f64).abs() < 10000.0); // Allow 10km tolerance
    }
}
