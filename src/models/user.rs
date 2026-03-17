use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Users{
    pub wallet_address: String,
    pub starting_balance: f64,
    pub current_balance: f64,
    pub total_realized_pnl: f64,
    pub created_at: i64,
}