use std::error::Error;

use crate::{AppState, PortfolioPerformance};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PortfolioPerformanceRequest {
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
}

#[derive(Serialize)]
pub struct PortfolioPerformanceResponse {
    pub history: Vec<PortfolioPerformance>,
    pub next_page_token: Option<String>,
}

const PAGE_SIZE: i32 = 20;
pub async fn get_performance_history(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
    Query(params): Query<PortfolioPerformanceRequest>,
) -> Result<Json<PortfolioPerformanceResponse>, (StatusCode, String)> {
    let page_size = params.page_size.unwrap_or(PAGE_SIZE);
    let (history, next_page_token) = state
        .portfolio_performance_service
        .get_performance_portfolio(wallet_address, page_size, params.page_token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PortfolioPerformanceResponse {
        history,
        next_page_token,
    }))
}
pub fn routes() -> Router<AppState> {
    Router::new().route("/{wallet_address}", get(get_performance_history))
}
