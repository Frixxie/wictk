pub async fn ping() -> &'static str {
    log::info!("GET /status/ping");
    "pong"
}

pub async fn health() -> &'static str {
    log::info!("GET /status/health");
    "OK"
}
