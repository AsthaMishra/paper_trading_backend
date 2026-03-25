use axum::{Router, response::IntoResponse, routing::get};
use paper_trading_backend::{AppConfig, AppState, config::DatabaseConfig, routes};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let app_config = AppConfig::from_env();
    let db_config = DatabaseConfig::from_env();

    let session = db_config.create_session().await?;
    let app_state = AppState::new(session).await?;

    let routes = Router::new()
        .route("/health", get(health_check))
        .nest("/users", routes::user::routes())
        .nest("/trade", routes::trade::routes())
        .nest("/portfolio", routes::portfolio::routes())
        .nest("/leaderboard", routes::leaderboard::routes())
        .nest("/portfolio-performance", routes::portfolio_performance::routes())
        .nest("/closed-positions", routes::closed_positions::routes());

    let app = Router::new()
        .merge(routes)
        .layer(CorsLayer::permissive())
        .fallback(handle_404)
        .with_state(app_state);

    let addr = format!("{}:{}", app_config.host, app_config.port);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}

pub async fn handle_404() -> impl IntoResponse {
    "Not found"
}
