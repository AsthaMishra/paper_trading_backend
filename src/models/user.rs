use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    pub wallet_address: String,
    pub starting_balance: f64,
    pub current_balance: f64,
    pub total_realized_pnl: f64,
    pub created_at: i64,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub best_trade: f64,
    pub worst_trade: f64,
}
