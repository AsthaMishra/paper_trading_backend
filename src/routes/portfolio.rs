use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
};
use serde::Deserialize;
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

#[derive(Deserialize)]
pub struct SetTriggersRequest {
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

pub async fn set_triggers(
    State(state): State<AppState>,
    Path((wallet_address, asset)): Path<(String, String)>,
    Json(req): Json<SetTriggersRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if req.stop_loss.is_none() && req.take_profit.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            "provide at least one of stop_loss or take_profit".to_string(),
        ));
    }
    state
        .portfolio_service
        .set_auto_sell(&wallet_address, &asset, req.stop_loss, req.take_profit)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}


pub fn routes() -> Router<AppState> {
    Router::new().route("/{wallet_address}", get(get_portfolio))
            .route("/{wallet_address}/{asset}/triggers", patch(set_triggers))

}
