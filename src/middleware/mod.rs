use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{AppError, AppState, auth::decode_jwt};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = decode_jwt(token, &state.jwt_secret)?;
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
