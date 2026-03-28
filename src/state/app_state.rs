use std::sync::Arc;

use scylla::client::session::Session;

use crate::{LeaderboardService, LimitOrderService, MarketDataService, PortfolioPerformanceService, PortfolioService, TradeService, UserService};


#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    pub trade_service: TradeService,
    pub portfolio_service: PortfolioService,
    pub leaderboard_service: LeaderboardService,
    pub portfolio_performance_service: PortfolioPerformanceService,
    pub limit_order_service: LimitOrderService,
    pub market_data_service: MarketDataService,
}

impl AppState {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn std::error::Error>> {
        let market_data_service = MarketDataService::new();

        // Pre-warm common assets so the cache is hot before the first request arrives.
        market_data_service.warm_up(&["SOL", "ETH", "BTC", "BNB"]).await;

        Ok(Self {
            user_service: UserService::new(db.clone()).await?,
            trade_service: TradeService::new(db.clone()).await?,
            portfolio_service: PortfolioService::new(db.clone()).await?,
            leaderboard_service: LeaderboardService::new(db.clone()).await?,
            portfolio_performance_service: PortfolioPerformanceService::new(db.clone()).await?,
            limit_order_service: LimitOrderService::new(db.clone()).await?,
            market_data_service,
        })
    }
}
