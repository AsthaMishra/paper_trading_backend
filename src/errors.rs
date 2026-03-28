use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 404 — resource (user, position, order) does not exist
    #[error("{0}")]
    NotFound(String),

    /// 401 — missing or invalid JWT
    #[error("unauthorized")]
    Unauthorized,

    /// 400 — caller-provided data is invalid
    #[error("{0}")]
    BadRequest(String),

    /// 502 — upstream market data call failed
    #[error("market data unavailable: {0}")]
    MarketData(String),

    /// 500 — database or other internal failure
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn internal(e: impl std::fmt::Display) -> Self {
        AppError::Internal(e.to_string())
    }

    pub fn market(e: impl std::fmt::Display) -> Self {
        AppError::MarketData(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::MarketData(m) => (StatusCode::BAD_GATEWAY, m.clone()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
