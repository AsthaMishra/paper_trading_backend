use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PortfolioPerformance {
    pub wallet_address: String,
    pub timestamp: i64,
    pub balance: f64,
    pub realized_pnl: f64,
}

