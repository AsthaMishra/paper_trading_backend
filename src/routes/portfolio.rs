use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::Serialize;

use crate::{AppState, Positions};

#[derive(Serialize)]
pub struct PositionWithUnrealizedPnl {
    pub wallet_address: String,
    pub asset: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub opened_at: i64,
    pub updated_at: i64,
}

impl PositionWithUnrealizedPnl {
    fn from_position(pos: Positions, current_price: f64) -> Self {
        let unrealized_pnl = (current_price - pos.avg_entry_price) * pos.quantity;
        Self {
            wallet_address: pos.wallet_address,
            asset: pos.asset,
            quantity: pos.quantity,
            avg_entry_price: pos.avg_entry_price,
            current_price,
            unrealized_pnl,
            realized_pnl: pos.realized_pnl,
            opened_at: pos.opened_at,
            updated_at: pos.updated_at,
        }
    }
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<Json<Vec<PositionWithUnrealizedPnl>>, (StatusCode, String)> {
    let positions = state
        .portfolio_service
        .get_positions(&wallet_address)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if positions.is_empty() {
        return Ok(Json(vec![]));
    }

    let assets: Vec<String> = positions.iter().map(|p| p.asset.clone()).collect();
    let prices = state
        .market_data_service
        .get_prices(&assets)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = positions
        .into_iter()
        .map(|pos| {
            let price = prices.get(&pos.asset).copied().unwrap_or(pos.avg_entry_price);
            PositionWithUnrealizedPnl::from_position(pos, price)
        })
        .collect();

    Ok(Json(result))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/{wallet_address}", get(get_portfolio))
}
