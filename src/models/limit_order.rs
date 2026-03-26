use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LimitOrder {
    pub wallet_address: String,
    pub id: Uuid,
    pub asset: String,
    pub side: String,
    pub order_type: String, // "buy_limit" | "stop_loss" | "take_profit"
    pub quantity: f64,
    pub limit_price: f64,
    pub created_at: i64,
}
