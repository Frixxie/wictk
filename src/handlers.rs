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
use reqwest::Client;

pub async fn ping() -> &'static str {
    log::info!("GET /status/ping");
    "pong"
}

pub async fn geocoding(client: Client, location: String) -> Option<Vec<OpenWeatherMapLocation>> {
    match client
        .get("https://api.openweathermap.org/geo/1.0/direct")
        .query(&[("q", location)])
        .query(&[("appid", env!("OPENWEATHERMAPAPIKEY"))])
        .send()
        .await
    {
        Ok(result) => match result.json::<Vec<OpenWeatherMapLocation>>().await {
            Ok(res) => Some(res),
            Err(err) => {
                log::error!("Error: {}", err);
                None
            }
        },
        Err(err) => {
            log::error!("Error: {}", err);
            None
        }
    }
}

pub async fn get_geocoding(
    State(client): State<Client>,
    Query(query): Query<City>,
) -> Result<Json<Vec<OpenWeatherMapLocation>>, InternalApplicationError> {
    log::info!("GET /api/geocoding");
    let res = geocoding(client, query.location).await.ok_or_else(|| {
        log::error!("Failed to get geocoding data from OpenWeatherMap");
        InternalApplicationError::new("Failed to get geocoding data from OpenWeatherMap")
    })?;
    Ok(Json(res))
}

pub async fn alerts(
    State(client): State<Client>,
) -> Result<Json<Vec<Alert>>, InternalApplicationError> {
    log::info!("GET /api/alerts");
    let alerts = MetAlert::fetch(client.clone(), Coordinates::new(59.91273, 10.74609))
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no alerts")
        })?;
    Ok(Json(alerts))
}

pub async fn nowcasts(
    State(client): State<Client>,
    Query(query): Query<LocationQuery>,
) -> Result<Json<Vec<Nowcast>>, InternalApplicationError> {
    log::info!("GET /api/nowcasts");
    let location = match query {
        LocationQuery::Location(loc_query) => {
            let res = geocoding(client.clone(), loc_query.location)
                .await
                .ok_or_else(|| {
                    InternalApplicationError::new(
                        "Failed to get geocoding data from OpenWeatherMap",
                    )
                })?;
            res.first()
                .ok_or_else(|| {
                    InternalApplicationError::new(
                        "Failed to get geocoding data from OpenWeatherMap",
                    )
                })?
                .location
                .clone()
        }
        LocationQuery::Coordinates(location) => location.try_into().map_err(|_| {
            InternalApplicationError::new("Failed to convert value to location type")
        })?,
    };

    let met_cast_handle = tokio::spawn(MetNowcast::fetch(client.clone(), location.clone()));
    let openweathermap_cast_handle =
        tokio::spawn(OpenWeatherNowcast::fetch(client.clone(), location.clone()));

    let met_cast = met_cast_handle
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no nowcast")
        })?
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get Met.no nowcast")
        })?;

    let openweathermap = openweathermap_cast_handle
        .await
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get OpenWeatherMap nowcast")
        })?
        .map_err(|err| {
            log::error!("Error {}", err);
            InternalApplicationError::new("Failed to get OpenWeatherMap nowcast")
        })?;

    let nowcasts: Vec<Nowcast> = vec![met_cast.into(), openweathermap.into()];
    Ok(Json(nowcasts))
}

#[cfg(test)]
mod tests {
    use axum::extract::Query;
    use http::Uri;
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
        let res = super::geocoding(client, "Oslo".to_string()).await;
        assert!(res.is_some());
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
        let res = super::alerts(axum::extract::State(client)).await;
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
            axum::extract::State(client.clone()),
            axum::extract::Query(super::LocationQuery::Coordinates(CoordinatesAsString {
                lat: "59.91273".to_string(),
                lon: "10.74609".to_string(),
            })),
        )
        .await;
        assert!(res.is_ok());
    }
}
