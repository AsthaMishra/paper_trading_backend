use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState, auth::encode_jwt};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub wallet_address: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub wallet_address: String,
    /// Seconds until expiry
    pub expires_in: u64,
}

/// POST /auth/login
/// Creates the user account (IF NOT EXISTS) and returns a signed JWT.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if req.wallet_address.trim().is_empty() {
        return Err(AppError::BadRequest("wallet_address is required".into()));
    }

    // Upsert: safe because the INSERT uses IF NOT EXISTS
    state
        .user_service
        .create_user(req.wallet_address.clone())
        .await
        .map_err(AppError::internal)?;

    let token = encode_jwt(&req.wallet_address, &state.jwt_secret)?;

    Ok(Json(LoginResponse {
        token,
        wallet_address: req.wallet_address,
        expires_in: 86_400,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/login", post(login))
}
