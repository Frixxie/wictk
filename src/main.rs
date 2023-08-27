mod weather;
mod handlers;
mod utils;

use axum::{
    routing::get,
    Router, Server,
};

use crate::handlers::{alerts, nowcasts};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_builder = reqwest::Client::builder();
    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        " ",
        env!("CARGO_PKG_HOMEPAGE"),
    );
    let client = client_builder.user_agent(APP_USER_AGENT).build().unwrap();

    let api = Router::new()
        .route("/alerts", get(alerts))
        .route("/nowcasts", get(nowcasts))
        .with_state(client);

    let app = Router::new().nest("/api", api);

    Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
