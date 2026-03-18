use tokio::net::TcpListener;
use axum::{Router, response::IntoResponse, routing::get};
use paper_trading_backend::{AppConfig, AppState, config::DatabaseConfig};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    dotenvy::dotenv().ok();

    let app_config = AppConfig::from_env();
    let db_confifg = DatabaseConfig::from_env();

    let session = db_confifg.create_session().await?;

    let app_state = AppState::new(session);

    let routes = Router::new().route("/health", get(health_check));

    let app = Router::new()
        .merge(routes)
        .layer(CorsLayer::permissive())
        .fallback(handle_404)
        .with_state(app_state);

    let addr = format!("{}:{}", app_config.host, app_config.port);
    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}

pub async fn handle_404() -> impl IntoResponse {
    "Not found"
}
