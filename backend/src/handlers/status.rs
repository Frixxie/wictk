pub async fn ping() -> &'static str {
    tracing::info!("GET /status/ping");
    "pong"
}

pub async fn health() -> &'static str {
    tracing::info!("GET /status/health");
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use crate::handlers::test_utils::{create_test_app, make_request};

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
