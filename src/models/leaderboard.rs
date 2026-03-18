use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Leaderboard {
    bucket: String,
    total_pnl: f64,
    wallet_address: String,
}
