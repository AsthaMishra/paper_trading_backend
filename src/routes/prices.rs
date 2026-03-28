use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct PricesQuery {
    pub tokens: String,
}

pub async fn get_prices(
    State(state): State<AppState>,
    Query(params): Query<PricesQuery>,
) -> Result<Json<HashMap<String, f64>>, (StatusCode, String)> {
    let tokens: Vec<String> = params
        .tokens
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tokens.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "tokens query param is required".to_string()));
    }

    state
        .market_data_service
        .get_prices(&tokens)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get_prices))
}
