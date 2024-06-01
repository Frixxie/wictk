use std::time::Duration;

use axum::{
    extract::{Query, State},
    Json,
};
use reqwest::StatusCode;
use tokio::time::Instant;

use crate::{
    cache::{Cache, TimedCache},
    locations::{City, OpenWeatherMapLocation},
    AppState,
};

use super::error::ApplicationError;

pub async fn lookup_location(
    client: &reqwest::Client,
    location: &str,
    loc_cache: &Cache<String, Option<OpenWeatherMapLocation>>,
) -> Result<OpenWeatherMapLocation, ApplicationError> {
    match loc_cache.get(location.to_string()).await {
        Some(location) => match location {
            Some(loc) => Ok(loc),
            None => Err(ApplicationError::new(
                "Location not found",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
        None => {
            let res = match OpenWeatherMapLocation::fetch(client, location).await {
                Some(locs) => {
                    if locs.is_empty() {
                        loc_cache
                            .set(
                                location.to_string(),
                                None,
                                Instant::now() + Duration::from_secs(300),
                            )
                            .await;
                        return Err(ApplicationError::new(
                            "Location not found",
                            StatusCode::NOT_FOUND,
                        ));
                    }
                    locs.first().unwrap().clone()
                }
                None => {
                    loc_cache
                        .set(
                            location.to_string(),
                            None,
                            Instant::now() + Duration::from_secs(300),
                        )
                        .await;
                    return Err(ApplicationError::new(
                        "Location not found",
                        StatusCode::NOT_FOUND,
                    ));
                }
            };
            loc_cache
                .set(
                    location.to_string(),
                    Some(res.clone()),
                    Instant::now() + Duration::from_secs(300),
                )
                .await;
            Ok(res)
        }
    }
}

pub async fn geocoding(
    State(app_state): State<AppState>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, ApplicationError> {
    let res = OpenWeatherMapLocation::fetch(&app_state.client, &query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            ApplicationError::new(
                "Failed to get geocoding data from OpenWeatherMap",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;
    Ok(Json(res))
}

#[cfg(test)]

mod tests {
    use crate::{cache::TimedCache, handlers::location::lookup_location};

    #[tokio::test]
    async fn test_lookup_location() {
        let client = reqwest::Client::new();
        let loc_cache = crate::Cache::new();
        let location = lookup_location(&client, "Oslo", &loc_cache).await.unwrap();
        assert_eq!(location.name, "Oslo");
    }

    #[tokio::test]
    async fn test_lookup_location_not_found() {
        let client = reqwest::Client::new();
        let loc_cache = crate::Cache::new();
        let location = lookup_location(&client, "Åkreham", &loc_cache).await;
        dbg!(&location);
        assert!(location.is_err());

        let loc = loc_cache.get("Åkreham".to_string()).await;
        dbg!(&loc);
        assert!(loc.is_some_and(|loc| loc.is_none()));
    }
}
