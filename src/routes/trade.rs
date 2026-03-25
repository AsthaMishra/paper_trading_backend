use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{AppState, Trades};
use super::DEFAULT_PAGE_SIZE;

#[derive(Deserialize)]
pub struct TradeRequest {
    pub wallet_address: String,
    pub asset: String,
    pub side: String,
    pub quantity: f64,
    pub order_price: f64,
}

#[derive(Serialize)]
pub struct TradeResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct TradeHistoryQuery {
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
}

#[derive(Serialize)]
pub struct TradeHistoryResponse {
    pub trades: Vec<Trades>,
    pub next_page_token: Option<String>,
}

pub async fn execute_trade(
    State(state): State<AppState>,
    Json(req): Json<TradeRequest>,
) -> Result<Json<TradeResponse>, (StatusCode, String)> {
    if req.quantity <= 0.0 {
        return Err((StatusCode::BAD_REQUEST, "quantity must be greater than 0".to_string()));
    }
    if req.order_price <= 0.0 {
        return Err((StatusCode::BAD_REQUEST, "order_price must be greater than 0".to_string()));
    }
    if req.asset.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "asset cannot be empty".to_string()));
    }

    let result = match req.side.as_str() {
        "buy" => {
            state
                .trade_service
                .buy(req.wallet_address, req.asset, req.quantity, req.order_price)
                .await
        }
        "sell" => {
            state
                .trade_service
                .sell(req.wallet_address, req.asset, req.quantity, req.order_price)
                .await
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "side must be 'buy' or 'sell'".to_string(),
            ));
        }
    };

    result.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(TradeResponse {
        message: format!("{} executed successfully", req.side),
    }))
}

pub async fn get_trades(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
    Query(params): Query<TradeHistoryQuery>,
) -> Result<Json<TradeHistoryResponse>, (StatusCode, String)> {
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

    let (trades, next_page_token) = state
        .trade_service
        .get_trades(&wallet_address, page_size, params.page_token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TradeHistoryResponse { trades, next_page_token }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(execute_trade))
        .route("/{wallet_address}", get(get_trades))
}
