#[utoipa::path(
    get,
    path = "/status/ping",
    responses(
        (status = 200, description = "Service is running", body = String)
    ),
    tag = "status"
)]
pub async fn ping() -> &'static str {
    tracing::info!("GET /status/ping");
    "pong"
}

#[utoipa::path(
    get,
    path = "/status/health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    ),
    tag = "status"
)]
pub async fn health() -> &'static str {
    tracing::info!("GET /status/health");
    "OK"
}

#[cfg(test)]
mod tests {
    use crate::handlers::test_utils::{create_test_app, make_request};
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_ping_endpoint() {
        let app = create_test_app();
        let (status, body) = make_request(app, "/status/ping").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(&body[..], b"pong");
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_test_app();
        let (status, body) = make_request(app, "/status/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(&body[..], b"OK");
    }
}
