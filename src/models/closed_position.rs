use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClosedPosition {
    pub wallet_address: String,
    pub closed_at: i64,
    pub asset: String,
    pub opened_at: i64,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub exit_price: f64,
    pub realized_pnl: f64,
}
