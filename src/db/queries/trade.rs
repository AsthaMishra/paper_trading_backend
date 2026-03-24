use std::error::Error;

use scylla::{
    client::session::Session, response::{PagingState, PagingStateResponse}, statement::prepared::PreparedStatement
};
use uuid::Uuid;

use crate::Trades;

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
    fees
) VALUES (?,?,?,?,?,?,?,?,?,?)";

const GET_TRADES: &str = "SELECT wallet_address, created_at, id, asset, side, quantity, order_price, filled_price, total_value, fees
    FROM paper_trading.trades WHERE wallet_address = ?";

#[derive(Clone)]
pub struct TradeDB {
    trade: PreparedStatement,
    get_trades: PreparedStatement,
}

impl TradeDB {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            trade: session.prepare(TRADE).await?,
            get_trades: session.prepare(GET_TRADES).await?,
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
        session
            .execute_unpaged(
                &self.trade,
                (
                    wallet_address,
                    chrono::Utc::now().timestamp_millis(),
                    id,
                    asset,
                    side,
                    quantity,
                    order_price,
                    filled_price,
                    total_value,
                    fees,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn get_trades(
        &self,
        session: &Session,
        wallet_address: &str,
        page_size: i32,
        paging_state: PagingState,
    ) -> Result<(Vec<Trades>, Option<PagingState>), Box<dyn Error>> {
        let mut statement = self.get_trades.clone();
        statement.set_page_size(page_size);

        let (result, paging_response) = session
            .execute_single_page(&statement, (wallet_address,), paging_state)
            .await?;

        let trades = result
            .into_rows_result()?
            .rows::<(String, i64, Uuid, String, String, f64, f64, f64, f64, f64)>()?
            .filter_map(|r| r.ok())
            .map(|(wallet_address, created_at, id, asset, side, quantity, order_price, filled_price, total_value, fees)| {
                Trades { wallet_address, created_at, id, asset, side, quantity, order_price, filled_price, total_value, fees }
            })
            .collect();

        let next_page: Option<PagingState> = match paging_response {
            PagingStateResponse::HasMorePages { state } => Some(state),
            PagingStateResponse::NoMorePages => None,
        };

        Ok((trades, next_page))
    }
}
