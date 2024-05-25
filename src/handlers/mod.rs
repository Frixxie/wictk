use crate::AppState;
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use log::info;
use tokio::time::Instant;
use tower::ServiceBuilder;

use self::{
    alerts::alerts,
    location::geocoding,
    nowcasts::nowcasts,
    status::{health, ping},
};

mod alerts;
mod error;
mod location;
mod nowcasts;
mod status;

pub use alerts::Alerts;

pub async fn profile_endpoint(request: Request, next: Next) -> Response {
    let method = request.method().clone().to_string();
    let uri = request.uri().clone();
    info!("Handling {} at {}", method, uri);

    let now = Instant::now();

    let response = next.run(request).await;

    let elapsed = now.elapsed();

    info!(
        "Finished handling {} at {}, used {} ms",
        method,
        uri,
        elapsed.as_millis()
    );
    response
}

pub fn setup_router(app_state: AppState) -> Router {
    let api = Router::new()
        .route("/alerts", get(alerts))
        .route("/nowcasts", get(nowcasts))
        .route("/geocoding", get(geocoding))
        .with_state(app_state)
        .layer(ServiceBuilder::new().layer(middleware::from_fn(profile_endpoint)));

    let status = Router::new()
        .route("/ping", get(ping))
        .route("/health", get(health));

    Router::new().nest("/status", status).nest("/api", api)
}

#[cfg(test)]
mod tests {
    use axum::{
        extract::{Query, State},
        http::Uri,
    };

    use crate::{
        handlers::nowcasts::{LocationQuery, ProviderQuery},
        locations::{City, CoordinatesAsString},
        AppState,
    };

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
        let app_state = AppState::new(client);
        let res = super::geocoding(
            State(app_state),
            Query(City {
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
        let app_state = AppState::new(client);

        let res = super::alerts(
            State(app_state),
            Query(City {
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
        let app_state = AppState::new(client);
        let res = super::nowcasts(
            State(app_state),
            Query(ProviderQuery { provider: None }),
            Query(LocationQuery::Coordinates(CoordinatesAsString {
                lat: "59.91273".to_string(),
                lon: "10.74609".to_string(),
            })),
        )
        .await;
        assert!(res.is_ok());
    }

    #[test]
    fn test_locationtype_city() {
        let json = r#"{"location": "Oslo"}"#;
        let location: LocationQuery = serde_json::from_str(json).unwrap();
        assert_eq!(
            location,
            LocationQuery::Location(City {
                location: "Oslo".to_string()
            })
        );
    }

    #[test]
    fn test_locationtype_coordinates_strings() {
        let json = r#"{"lat": "1.0", "lon": "2.0"}"#;
        let location: LocationQuery = serde_json::from_str(json).unwrap();
        assert_eq!(
            location,
            LocationQuery::Coordinates(CoordinatesAsString {
                lat: "1.0".to_string(),
                lon: "2.0".to_string()
            })
        );
    }
}
