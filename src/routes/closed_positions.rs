use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use scylla::response::PagingState;
use serde::{Deserialize, Serialize};

use crate::{AppState, ClosedPosition};
use super::DEFAULT_PAGE_SIZE;

#[derive(Deserialize)]
pub struct ClosedPositionsQuery {
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
}

#[derive(Serialize)]
pub struct ClosedPositionsResponse {
    pub positions: Vec<ClosedPosition>,
    pub next_page_token: Option<String>,
}

pub async fn get_closed_positions(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
    Query(params): Query<ClosedPositionsQuery>,
) -> Result<Json<ClosedPositionsResponse>, (StatusCode, String)> {
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let paging_state = match params.page_token {
        Some(token) => {
            let bytes = BASE64_STANDARD
                .decode(token)
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            PagingState::new_from_raw_bytes(bytes)
        }
        None => PagingState::start(),
    };

    let (positions, next_state) = state
        .portfolio_service
        .get_closed_positions(&wallet_address, page_size, paging_state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let next_page_token = next_state.and_then(|s| {
        s.as_bytes_slice().map(|b| BASE64_STANDARD.encode(b.as_ref()))
    });

    Ok(Json(ClosedPositionsResponse { positions, next_page_token }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/{wallet_address}", get(get_closed_positions))
}
