use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;

use crate::{AppState, Leaderboard};

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub bucket: Option<String>,
    pub limit: Option<i32>,
}

pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<Vec<Leaderboard>>, (StatusCode, String)> {
    let bucket = params.bucket.as_deref().unwrap_or("global");
    let limit = params.limit.unwrap_or(10);

    state
        .leaderboard_service
        .get_top(bucket, limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("leaderboard", get(get_leaderboard))
}
