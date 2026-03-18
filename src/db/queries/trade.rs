use std::error::Error;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};
use uuid::Uuid;

const TRADE: &str = "INSERT INTO paper_trading.trades (
    wallet_address,
    created_at,
    id,
    asset,
    side,
    quantity,
    order_price,
    filled_price,
    total_value,
    fees,
) VALUES (?,?,?,?,?,?,?,?,?,?)";

pub struct TradeDB {
    trade: PreparedStatement,
}

impl TradeDB {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            trade: session.prepare(TRADE).await?,
        })
    }

    pub async fn record(
        &self,
        session: &Session,
        wallet_address: String,
        id: Uuid,
        asset: String,
        side: String,
        quantity: f64,
        order_price: f64,
        filled_price: f64,
        total_value: f64,
        fees: f64,
    ) -> Result<(), Box<dyn Error>> {
        session.execute_unpaged(&self.trade, (
            wallet_address,
            chrono::Utc::now().timestamp(),
            id,
            asset,
            side,
            quantity,
            order_price,
            filled_price,
            total_value,
            fees
        )).await?;
        Ok(())
    }
}
