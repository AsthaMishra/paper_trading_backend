use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{AppState, Users};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub wallet_address: String,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub message: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, String)> {
    state
        .user_service
        .create_user(req.wallet_address)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CreateUserResponse {
        message: "User created successfully".to_string(),
    }))
}

#[derive(Serialize)]
pub struct GetUserResponse {
    pub user: Option<Users>,
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<Json<GetUserResponse>, (StatusCode, String)> {
    let user = state
        .user_service
        .get_user(&wallet_address)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(GetUserResponse { user }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/:wallet_address", get(get_user))
}
