use crate::AppState;
use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use lightning::get_recent_lightning;
use metrics::histogram;
use metrics_exporter_prometheus::PrometheusHandle;
use nowcasts::{nowcast_met, nowcast_openweathermap, nowcasts};
use tokio::time::Instant;
use tower::ServiceBuilder;
use tracing::{info, instrument};

use self::{
    alerts::alerts,
    location::geocoding,
    status::{health, ping},
};

mod alerts;
mod error;
mod lightning;
mod location;
mod nowcasts;
mod status;

#[cfg(test)]
mod test_utils;

pub use alerts::Alerts;

#[instrument]
pub async fn profile_endpoint(request: Request, next: Next) -> Response {
    let method = request.method().clone().to_string();
    let uri = request.uri().clone().to_string();

    info!("Handling {} at {}", method, uri);

    let now = Instant::now();

    let response = next.run(request).await;

    let elapsed = now.elapsed();

    let labels = [("method", method.clone()), ("uri", uri.clone())];

    histogram!("handler", &labels).record(elapsed);

    info!(
        "Finished handling {} at {}, used {} ms",
        method,
        uri,
        elapsed.as_millis()
    );
    response
}

pub fn setup_router(app_state: AppState, metrics_handler: PrometheusHandle) -> Router {
    let api = Router::new()
        .route("/alerts", get(alerts))
        .route("/owm/nowcasts", get(nowcast_openweathermap))
        .route("/met/nowcasts", get(nowcast_met))
        .route("/nowcasts", get(nowcasts))
        .route("/geocoding", get(geocoding))
        .route("/recent_lightning", get(get_recent_lightning))
        .with_state(app_state);

    let status = Router::new()
        .route("/ping", get(ping))
        .route("/health", get(health));

    Router::new()
        .route("/metrics", get(metrics))
        .with_state(metrics_handler)
        .nest("/status", status)
        .nest("/api", api)
        .layer(ServiceBuilder::new().layer(middleware::from_fn(profile_endpoint)))
}

#[instrument]
async fn metrics(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}

#[cfg(test)]
mod tests {
    use axum::{extract::Query, http::Uri};
    use wictk_core::{City, CoordinatesAsString};
    use axum::http::StatusCode;
    use crate::handlers::test_utils::{create_test_app, make_request, make_request_with_method};

    use crate::handlers::nowcasts::LocationQuery;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = create_test_app();
        let (status, _body) = make_request(app, "/metrics").await;
        
        assert_eq!(status, StatusCode::OK);
        
        // Metrics endpoint may return empty if no metrics have been recorded yet
        // The important thing is that it responds with 200 OK
    }

    #[tokio::test]
    async fn test_invalid_endpoint() {
        let app = create_test_app();
        let (status, _body) = make_request(app, "/api/invalid").await;
        
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_invalid_method() {
        let app = create_test_app();
        let (status, _body) = make_request_with_method(app, "POST", "/status/ping").await;
        
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_endpoint_timing_metrics() {
        let app = create_test_app();
        
        // Make a request to generate metrics
        let _ = make_request(app.clone(), "/status/ping").await;
        
        // Check that metrics are generated
        let (status, body) = make_request(app, "/metrics").await;
        assert_eq!(status, StatusCode::OK);
        
        let body_str = String::from_utf8(body).unwrap();
        assert!(body_str.contains("handler"));
    }

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
