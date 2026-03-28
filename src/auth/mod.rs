use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::AppError;

const TOKEN_TTL_SECS: i64 = 86_400; // 24 hours

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    pub fn wallet(&self) -> &str {
        &self.sub
    }
}

pub fn encode_jwt(wallet: &str, secret: &str) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: wallet.to_string(),
        iat: now,
        exp: now + TOKEN_TTL_SECS,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("failed to sign token: {e}")))
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|td| td.claims)
    .map_err(|e| {
        log::debug!("JWT decode failed: {e}");
        AppError::Unauthorized
    })
}
