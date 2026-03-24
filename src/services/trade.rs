use std::{error::Error, sync::Arc};

use base64::{Engine, prelude::BASE64_STANDARD};
use scylla::{client::session::Session, response::PagingState};
use uuid::Uuid;

use crate::{Leaderboard, LeaderboardDb, PositionsDb, TradeDB, UserDb};

const FEE_RATE: f64 = 0.001; // 0.1%

#[derive(Clone)]
pub struct TradeService {
    db: Arc<Session>,
    user_db: UserDb,
    positions_db: PositionsDb,
    trade_db: TradeDB,
    leaderboard_db: LeaderboardDb,
}

impl TradeService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            user_db: UserDb::new(&db).await?,
            positions_db: PositionsDb::new(&db).await?,
            trade_db: TradeDB::new(&db).await?,
            leaderboard_db: LeaderboardDb::new(&db).await?,
            db,
        })
    }

    pub async fn buy(
        &self,
        wallet_address: String,
        asset: String,
        quantity: f64,
        order_price: f64,
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

        match existing {
            None => {
                self.positions_db
                    .create_position(&self.db, &wallet_address, &asset, quantity, filled_price)
                    .await?;
            }
            Some(pos) => {
                let new_qty = pos.quantity + quantity;
                let new_avg =
                    (pos.quantity * pos.avg_entry_price + quantity * filled_price) / new_qty;
                self.positions_db
                    .update_position(&self.db, &wallet_address, &asset, new_qty, new_avg)
                    .await?;
            }
        }

        self.trade_db
            .record(
                &self.db,
                wallet_address.clone(),
                Uuid::new_v4(),
                asset,
                "buy".to_string(),
                quantity,
                order_price,
                filled_price,
                total_value,
                fees,
            )
            .await?;

        self.user_db
            .update_balance(&self.db, &wallet_address, user.current_balance - cost)
            .await?;

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
        let new_realized_pnl = pos.realized_pnl + pnl_this_trade;

        if (pos.quantity - quantity).abs() < f64::EPSILON {
            self.positions_db
                .full_sell(&self.db, &wallet_address, &asset)
                .await?;
        } else {
            let new_qty = pos.quantity - quantity;
            self.positions_db
                .partial_sell(&self.db, &wallet_address, &asset, new_qty, new_realized_pnl)
                .await?;
        }

        self.trade_db
            .record(
                &self.db,
                wallet_address.clone(),
                Uuid::new_v4(),
                asset.clone(),
                "sell".to_string(),
                quantity,
                order_price,
                filled_price,
                total_value,
                fees,
            )
            .await?;

        let new_balance = user.current_balance + total_value - fees;
        let new_total_pnl = user.total_realized_pnl + pnl_this_trade;

        self.user_db
            .update_balance(&self.db, &wallet_address, new_balance)
            .await?;

        self.user_db
            .update_pnl(&self.db, &wallet_address, new_total_pnl)
            .await?;

        self.leaderboard_db
            .upsert(
                &self.db,
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
