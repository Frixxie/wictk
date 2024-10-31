pub async fn ping() -> &'static str {
    tracing::info!("GET /status/ping");
    "pong"
}

pub async fn health() -> &'static str {
    tracing::info!("GET /status/health");
    "OK"
}
