use crate::{
    alerts::{Alert, AlertFetcher, MetAlert},
    location::{City, Coordinates, LocationQuery, OpenWeatherMapLocation},
    nowcasts::{MetNowcast, Nowcast, NowcastFetcher, OpenWeatherNowcast},
    utils::InternalApplicationError,
};
use axum::{
    extract::{Query, State},
    Json,
};
use log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn ping() -> &'static str {
    log::info!("GET /status/ping");
    "pong"
}

pub async fn geocoding(
    State(client): State<Client>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, InternalApplicationError> {
    log::info!("GET /api/geocoding");
    let res = OpenWeatherMapLocation::fetch(&client, &query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
    Ok(Json(res))
}

pub async fn alerts(
    State(client): State<Client>,
    Query(query): Query<City>,
) -> Result<Json<Vec<Alert>>, InternalApplicationError> {
    log::info!("GET /api/alerts");
    let res = OpenWeatherMapLocation::fetch(&client, &query.location)
        .await
        .ok_or_else(|| {
            log::error!("Failed to get geocoding data from OpenWeatherMap");
            InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
        })?;
    let alerts = MetAlert::fetch(client.clone(), res.first().unwrap().clone().location)
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no alerts")
        })?;
    Ok(Json(alerts))
}

#[derive(Serialize, Deserialize)]
pub enum NowcastProvider {
    Met,
    OpenWeatherMap,
}

#[derive(Serialize, Deserialize)]
pub struct NowcastQuery {
    provider: Option<NowcastProvider>,
}

async fn fetch_from_provider<T>(
    client: &Client,
    location: &Coordinates,
) -> Result<Nowcast, InternalApplicationError>
where
    T: NowcastFetcher,
{
    T::fetch(client, location).await.map_err(|err| {
        error!("Error {}", err);
        InternalApplicationError::new("Failed to get Met.no nowcast")
    })
}

async fn find_location(location: LocationQuery, client: &Client) -> anyhow::Result<Coordinates> {
    match location {
        LocationQuery::Location(city) => {
            let res = OpenWeatherMapLocation::fetch(&client, &city.location).await;
            let location = res.ok_or_else(|| {
                InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
            })?;
            Ok(location.first().unwrap().location.clone())
        }
        LocationQuery::Coordinates(cords_as_string) => {
            let cords = cords_as_string.try_into()?;
            Ok(cords)
        }
    }
}

pub async fn nowcasts(
    State(client): State<Client>,
    Query(query): Query<NowcastQuery>,
    Query(location): Query<LocationQuery>,
) -> Result<Json<Vec<Nowcast>>, InternalApplicationError> {
    log::info!("GET /api/nowcasts");
    let location = find_location(location, &client).await.map_err(|err| {
        error!("Error {}", err);
        InternalApplicationError::new("Failed to get location data")
    })?;

    let casts = match query.provider {
        Some(provider) => match provider {
            NowcastProvider::Met => {
                vec![fetch_from_provider::<MetNowcast>(&client, &location).await]
            }
            NowcastProvider::OpenWeatherMap => {
                vec![fetch_from_provider::<OpenWeatherNowcast>(&client, &location).await]
            }
        },
        None => vec![
            fetch_from_provider::<MetNowcast>(&client, &location).await,
            fetch_from_provider::<OpenWeatherNowcast>(&client, &location).await,
        ],
    };

    let nowcasts: Vec<Nowcast> = casts.into_iter().filter_map(|res| res.ok()).collect();

    Ok(Json(nowcasts))
}

#[cfg(test)]
mod tests {
    use axum::extract::{Query, State};
    use axum::http::Uri;
    use pretty_assertions::assert_eq;

    use crate::location::{City, CoordinatesAsString, LocationQuery};

    #[test]
    fn parse_location() {
        let uri: Uri = "http://localhost:3000/api/nowcasts?location=Oslo"
            .parse()
            .unwrap();

        let query = Query::<LocationQuery>::try_from_uri(&uri).unwrap();

        assert_eq!(
            query.0,
            LocationQuery::Location(City {
                location: "Oslo".to_string()
            })
        );
    }

    #[test]
    fn parse_coordinates() {
        let uri: Uri = "http://localhost:3000/api/nowcasts?lat=59.91273&lon=10.74609"
            .parse()
            .unwrap();

        let query = Query::<LocationQuery>::try_from_uri(&uri).unwrap();

        assert_eq!(
            query.0,
            LocationQuery::Coordinates(CoordinatesAsString {
                lat: "59.91273".to_string(),
                lon: "10.74609".to_string()
            })
        );
    }

    #[tokio::test]
    async fn get_geocoding() {
        let client = reqwest::Client::new();
        let res = super::geocoding(
            State(client.clone()),
            Query(super::City {
                location: "Oslo".to_string(),
            }),
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_alerts() {
        let client_builder = reqwest::Client::builder();
        static APP_USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
        let res = super::alerts(
            State(client.clone()),
            Query(super::City {
                location: "Oslo".to_string(),
            }),
        )
        .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn get_nowcasts() {
        let client_builder = reqwest::Client::builder();
        static APP_USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();
        let res = super::nowcasts(
            State(client.clone()),
            Query(super::NowcastQuery { provider: None }),
            Query(super::LocationQuery::Coordinates(CoordinatesAsString {
                lat: "59.91273".to_string(),
                lon: "10.74609".to_string(),
            })),
        )
        .await;
        assert!(res.is_ok());
    }
}
