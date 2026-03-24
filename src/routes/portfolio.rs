use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};

use crate::{AppState, Positions};

pub async fn get_portfolio(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<Json<Vec<Positions>>, (StatusCode, String)> {
    state
        .portfolio_service
        .get_positions(&wallet_address)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("portfolio", get(get_portfolio))
}
