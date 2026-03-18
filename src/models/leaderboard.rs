use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Leaderboard {
    pub bucket: String,
    pub total_pnl: f64,
    pub wallet_address: String,
}
