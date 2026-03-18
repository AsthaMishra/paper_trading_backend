use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Positions {
    pub wallet_address: String,
    pub asset: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub realized_pnl: f64,
    pub opened_at: i64,
    pub updated_at: i64,
}
