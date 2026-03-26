use std::{error::Error, sync::Arc};

use base64::{Engine, prelude::BASE64_STANDARD};
use scylla::{client::session::Session, response::PagingState};

use crate::{FullSellExtra, Leaderboard, LeaderboardDb, LimitOrder, LimitOrdersDb, PositionsDb, TradeDB, TradeData, UserDb};

const FEE_RATE: f64 = 0.001; // 0.1%

#[derive(Clone)]
pub struct TradeService {
    db: Arc<Session>,
    user_db: UserDb,
    positions_db: PositionsDb,
    trade_db: TradeDB,
    leaderboard_db: LeaderboardDb,
    limit_orders_db: LimitOrdersDb,
}

impl TradeService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            user_db: UserDb::new(&db).await?,
            positions_db: PositionsDb::new(&db).await?,
            trade_db: TradeDB::new(&db).await?,
            leaderboard_db: LeaderboardDb::new(&db).await?,
            limit_orders_db: LimitOrdersDb::new(&db).await?,
            db,
        })
    }

    pub async fn buy(
        &self,
        wallet_address: String,
        asset: String,
        quantity: f64,
        order_price: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<(), Box<dyn Error>> {
        let filled_price = order_price;
        let total_value = quantity * filled_price;
        let fees = total_value * FEE_RATE;
        let cost = total_value + fees;

        let user = self
            .user_db
            .get_user(&self.db, &wallet_address)
            .await?
            .ok_or("User not found")?;

        if user.current_balance < cost {
            return Err("Insufficient balance".into());
        }

        let existing = self
            .positions_db
            .get_position(&self.db, &wallet_address, &asset)
            .await?;

        let data = TradeData {
            wallet_address: &wallet_address,
            asset: &asset,
            quantity,
            filled_price,
            order_price,
            total_value,
            fees,
            new_balance: user.current_balance - cost,
            total_realized_pnl: user.total_realized_pnl,
            total_trades: user.total_trades + 1,
            winning_trades: user.winning_trades,
            best_trade: user.best_trade,
            worst_trade: user.worst_trade,
        };

        match existing {
            None => self.trade_db.buy_new_position(&self.db, &data).await?,
            Some(pos) => {
                let new_qty = pos.quantity + quantity;
                let new_avg =
                    (pos.quantity * pos.avg_entry_price + quantity * filled_price) / new_qty;
                self.trade_db
                    .buy_existing_position(&self.db, &data, new_qty, new_avg)
                    .await?;
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        if let Some(sl) = stop_loss {
            self.limit_orders_db.insert(&self.db, &LimitOrder {
                wallet_address: wallet_address.clone(),
                id: uuid::Uuid::new_v4(),
                asset: asset.clone(),
                side: "sell".to_string(),
                order_type: "stop_loss".to_string(),
                quantity,
                limit_price: sl,
                created_at: now,
            }).await?;
        }
        if let Some(tp) = take_profit {
            self.limit_orders_db.insert(&self.db, &LimitOrder {
                wallet_address: wallet_address.clone(),
                id: uuid::Uuid::new_v4(),
                asset: asset.clone(),
                side: "sell".to_string(),
                order_type: "take_profit".to_string(),
                quantity,
                limit_price: tp,
                created_at: now,
            }).await?;
        }

        Ok(())
    }

    pub async fn sell(
        &self,
        wallet_address: String,
        asset: String,
        quantity: f64,
        order_price: f64,
    ) -> Result<(), Box<dyn Error>> {
        let filled_price = order_price;
        let total_value = quantity * filled_price;
        let fees = total_value * FEE_RATE;

        let user = self
            .user_db
            .get_user(&self.db, &wallet_address)
            .await?
            .ok_or("User not found")?;

        let pos = self
            .positions_db
            .get_position(&self.db, &wallet_address, &asset)
            .await?
            .ok_or("Position not found")?;

        if pos.quantity < quantity {
            return Err("Insufficient position quantity".into());
        }

        let pnl_this_trade = (filled_price - pos.avg_entry_price) * quantity;
        let new_position_pnl = pos.realized_pnl + pnl_this_trade;
        let new_total_pnl = user.total_realized_pnl + pnl_this_trade;
        let new_winning_trades = if pnl_this_trade > 0.0 {
            user.winning_trades + 1
        } else {
            user.winning_trades
        };
        let is_first_sell =
            user.total_realized_pnl == 0.0 && user.best_trade == 0.0 && user.worst_trade == 0.0;
        let new_best_trade = if is_first_sell {
            pnl_this_trade
        } else {
            f64::max(user.best_trade, pnl_this_trade)
        };
        let new_worst_trade = if is_first_sell {
            pnl_this_trade
        } else {
            f64::min(user.worst_trade, pnl_this_trade)
        };

        let data = TradeData {
            wallet_address: &wallet_address,
            asset: &asset,
            quantity,
            filled_price,
            order_price,
            total_value,
            fees,
            new_balance: user.current_balance + total_value - fees,
            total_realized_pnl: new_total_pnl,
            total_trades: user.total_trades + 1,
            winning_trades: new_winning_trades,
            best_trade: new_best_trade,
            worst_trade: new_worst_trade,
        };

        if (pos.quantity - quantity).abs() < f64::EPSILON {
            let extra = FullSellExtra {
                opened_at: pos.opened_at,
                position_qty: pos.quantity,
                avg_entry_price: pos.avg_entry_price,
            };
            self.trade_db
                .sell_full(&self.db, &data, new_position_pnl, &extra)
                .await?;
        } else {
            let new_qty = pos.quantity - quantity;
            self.trade_db
                .sell_partial(&self.db, &data, new_qty, new_position_pnl)
                .await?;
        }

        self.leaderboard_db
            .upsert(
                &self.db,
                user.total_realized_pnl,
                Leaderboard {
                    bucket: "global".to_string(),
                    total_pnl: new_total_pnl,
                    wallet_address: wallet_address.clone(),
                },
            )
            .await?;

        Ok(())
    }

    pub async fn get_trades(
        &self,
        wallet_address: &str,
        page_size: i32,
        page_token: Option<String>,
    ) -> Result<(Vec<crate::Trades>, Option<String>), Box<dyn Error>> {
        let paging_state = match page_token {
            Some(token) => {
                let bytes = BASE64_STANDARD.decode(token)?;
                PagingState::new_from_raw_bytes(bytes)
            }
            None => PagingState::start(),
        };

        let (trades, next_state) = self
            .trade_db
            .get_trades(&self.db, wallet_address, page_size, paging_state)
            .await?;

        let next_token: Option<String> = next_state.and_then(|s| {
            s.as_bytes_slice()
                .map(|b| BASE64_STANDARD.encode(b.as_ref()))
        });

        Ok((trades, next_token))
    }
}
