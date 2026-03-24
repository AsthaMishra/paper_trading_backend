use std::sync::Arc;

use scylla::client::session::Session;

use crate::{LeaderboardService, PortfolioPerformanceService, PortfolioService, TradeService, UserService};


#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    pub trade_service: TradeService,
    pub portfolio_service: PortfolioService,
    pub leaderboard_service: LeaderboardService,
    pub portfolio_performance_service: PortfolioPerformanceService,
}

impl AppState {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            user_service: UserService::new(db.clone()).await?,
            trade_service:TradeService::new(db.clone()).await?,
            portfolio_service:PortfolioService::new(db.clone()).await?,
            leaderboard_service:LeaderboardService::new(db.clone()).await?,
            portfolio_performance_service: PortfolioPerformanceService::new(db.clone()).await?,
        })
    }
}
