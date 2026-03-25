use std::error::Error;

use scylla::{
    client::session::Session,
    response::{PagingState, PagingStateResponse},
    statement::batch::{Batch, BatchType},
};
use uuid::Uuid;

use crate::Trades;

use super::{
    closed_positions_db::INSERT_CLOSED_POSITION,
    portfolio_performance_db::SNAPSHOT,
    positions_db::{CREATE_POSITION, FULL_SELL, PARTIAL_SELL, UPDATE_POSITION},
    users_db::UPDATE_USER_QUERY,
};

pub(crate) const TRADE: &str = "INSERT INTO paper_trading.trades (
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

pub struct TradeData<'a> {
    pub wallet_address: &'a str,
    pub asset: &'a str,
    pub quantity: f64,
    pub filled_price: f64,
    pub order_price: f64,
    pub total_value: f64,
    pub fees: f64,
    pub new_balance: f64,
    pub total_realized_pnl: f64,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub best_trade: f64,
    pub worst_trade: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

pub struct FullSellExtra {
    pub opened_at: i64,
    pub position_qty: f64,
    pub avg_entry_price: f64,
}

#[derive(Clone)]
pub struct TradeDB {
    buy_new_batch: Batch,
    buy_existing_batch: Batch,
    sell_partial_batch: Batch,
    sell_full_batch: Batch,
    get_trades: scylla::statement::prepared::PreparedStatement,
}

impl TradeDB {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        let create_position = session.prepare(CREATE_POSITION).await?;
        let update_position = session.prepare(UPDATE_POSITION).await?;
        let partial_sell = session.prepare(PARTIAL_SELL).await?;
        let full_sell = session.prepare(FULL_SELL).await?;
        let insert_trade = session.prepare(TRADE).await?;
        let update_user = session.prepare(UPDATE_USER_QUERY).await?;
        let insert_closed = session.prepare(INSERT_CLOSED_POSITION).await?;
        let snapshot = session.prepare(SNAPSHOT).await?;

        let mut buy_new_batch = Batch::new(BatchType::Logged);
        buy_new_batch.append_statement(create_position);
        buy_new_batch.append_statement(insert_trade.clone());
        buy_new_batch.append_statement(update_user.clone());

        let mut buy_existing_batch = Batch::new(BatchType::Logged);
        buy_existing_batch.append_statement(update_position);
        buy_existing_batch.append_statement(insert_trade.clone());
        buy_existing_batch.append_statement(update_user.clone());

        let mut sell_partial_batch = Batch::new(BatchType::Logged);
        sell_partial_batch.append_statement(partial_sell);
        sell_partial_batch.append_statement(insert_trade.clone());
        sell_partial_batch.append_statement(update_user.clone());
        sell_partial_batch.append_statement(snapshot.clone());

        let mut sell_full_batch = Batch::new(BatchType::Logged);
        sell_full_batch.append_statement(insert_closed);
        sell_full_batch.append_statement(full_sell);
        sell_full_batch.append_statement(insert_trade);
        sell_full_batch.append_statement(update_user);
        sell_full_batch.append_statement(snapshot);

        Ok(Self {
            buy_new_batch,
            buy_existing_batch,
            sell_partial_batch,
            sell_full_batch,
            get_trades: session.prepare(GET_TRADES).await?,
        })
    }

    pub async fn buy_new_position(
        &self,
        session: &Session,
        d: &TradeData<'_>,
    ) -> Result<(), Box<dyn Error>> {
        let now = chrono::Utc::now().timestamp_millis();
        let id = Uuid::new_v4();
        let values = (
            (
                d.wallet_address,
                d.asset,
                d.quantity,
                d.filled_price,
                0.0f64,
                now,
                now,
                d.stop_loss,
                d.take_profit,
            ),
            (
                d.wallet_address,
                now,
                id,
                d.asset,
                "buy",
                d.quantity,
                d.order_price,
                d.filled_price,
                d.total_value,
                d.fees,
            ),
            (
                d.new_balance,
                d.total_realized_pnl,
                d.total_trades,
                d.winning_trades,
                d.best_trade,
                d.worst_trade,
                d.wallet_address,
            ),
        );
        session.batch(&self.buy_new_batch, values).await?;
        Ok(())
    }

    pub async fn buy_existing_position(
        &self,
        session: &Session,
        d: &TradeData<'_>,
        new_qty: f64,
        new_avg: f64,
    ) -> Result<(), Box<dyn Error>> {
        let now = chrono::Utc::now().timestamp_millis();
        let id = Uuid::new_v4();
        let values = (
            (
                new_qty,
                new_avg,
                now,
                d.stop_loss,
                d.take_profit,
                d.wallet_address,
                d.asset,
            ),
            (
                d.wallet_address,
                now,
                id,
                d.asset,
                "buy",
                d.quantity,
                d.order_price,
                d.filled_price,
                d.total_value,
                d.fees,
            ),
            (
                d.new_balance,
                d.total_realized_pnl,
                d.total_trades,
                d.winning_trades,
                d.best_trade,
                d.worst_trade,
                d.wallet_address,
            ),
        );
        session.batch(&self.buy_existing_batch, values).await?;
        Ok(())
    }

    pub async fn sell_partial(
        &self,
        session: &Session,
        d: &TradeData<'_>,
        new_qty: f64,
        new_position_pnl: f64,
    ) -> Result<(), Box<dyn Error>> {
        let now = chrono::Utc::now().timestamp_millis();
        let id = Uuid::new_v4();
        let values = (
            (new_qty, new_position_pnl, now, d.wallet_address, d.asset),
            (
                d.wallet_address,
                now,
                id,
                d.asset,
                "sell",
                d.quantity,
                d.order_price,
                d.filled_price,
                d.total_value,
                d.fees,
            ),
            (
                d.new_balance,
                d.total_realized_pnl,
                d.total_trades,
                d.winning_trades,
                d.best_trade,
                d.worst_trade,
                d.wallet_address,
            ),
            (d.wallet_address, now, d.new_balance, d.total_realized_pnl),
        );
        session.batch(&self.sell_partial_batch, values).await?;
        Ok(())
    }

    pub async fn sell_full(
        &self,
        session: &Session,
        d: &TradeData<'_>,
        new_position_pnl: f64,
        extra: &FullSellExtra,
    ) -> Result<(), Box<dyn Error>> {
        let now = chrono::Utc::now().timestamp_millis();
        let id = Uuid::new_v4();
        let values = (
            (
                d.wallet_address,
                now,
                d.asset,
                extra.opened_at,
                extra.position_qty,
                extra.avg_entry_price,
                d.filled_price,
                new_position_pnl,
            ),
            (d.wallet_address, d.asset),
            (
                d.wallet_address,
                now,
                id,
                d.asset,
                "sell",
                d.quantity,
                d.order_price,
                d.filled_price,
                d.total_value,
                d.fees,
            ),
            (
                d.new_balance,
                d.total_realized_pnl,
                d.total_trades,
                d.winning_trades,
                d.best_trade,
                d.worst_trade,
                d.wallet_address,
            ),
            (d.wallet_address, now, d.new_balance, d.total_realized_pnl),
        );
        session.batch(&self.sell_full_batch, values).await?;
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
            .map(
                |(
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
                )| {
                    Trades {
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
                    }
                },
            )
            .collect();

        let next_page: Option<PagingState> = match paging_response {
            PagingStateResponse::HasMorePages { state } => Some(state),
            PagingStateResponse::NoMorePages => None,
        };

        Ok((trades, next_page))
    }
}
