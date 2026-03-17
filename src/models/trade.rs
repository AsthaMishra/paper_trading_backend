use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Trades {
    pub wallet_address: String,
    pub created_at: i64,
    pub id: Uuid,
    pub asset: String,
    pub side: String,
    pub quantity: f64,
    pub order_price: f64,
    pub filled_price: f64,
    pub total_value: f64,
    pub fees: f64,
}
