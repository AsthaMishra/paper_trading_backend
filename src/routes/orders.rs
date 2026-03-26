use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, LimitOrder};

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub wallet_address: String,
    pub asset: String,
    pub side: String,
    pub order_type: String,
    pub quantity: f64,
    pub limit_price: f64,
}

pub async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<LimitOrder>, (StatusCode, String)> {
    if req.quantity <= 0.0 {
        return Err((StatusCode::BAD_REQUEST, "quantity must be greater than 0".to_string()));
    }
    if req.limit_price <= 0.0 {
        return Err((StatusCode::BAD_REQUEST, "limit_price must be greater than 0".to_string()));
    }
    let valid_side = matches!(req.side.as_str(), "buy" | "sell");
    if !valid_side {
        return Err((StatusCode::BAD_REQUEST, "side must be 'buy' or 'sell'".to_string()));
    }
    let valid_type = matches!(req.order_type.as_str(), "buy_limit" | "stop_loss" | "take_profit");
    if !valid_type {
        return Err((StatusCode::BAD_REQUEST, "order_type must be 'buy_limit', 'stop_loss', or 'take_profit'".to_string()));
    }

    state
        .limit_order_service
        .create(req.wallet_address, req.asset, req.side, req.order_type, req.quantity, req.limit_price)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_orders(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<Json<Vec<LimitOrder>>, (StatusCode, String)> {
    state
        .limit_order_service
        .get_orders(&wallet_address)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Path((wallet_address, id)): Path<(String, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .limit_order_service
        .cancel(&wallet_address, id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_order))
        .route("/{wallet_address}", get(get_orders))
        .route("/{wallet_address}/{id}", delete(cancel_order))
}
