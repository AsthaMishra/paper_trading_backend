use std::error::Error;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};

use crate::Users;

const CREATE_USER_QUERY: &str = "INSERT INTO paper_trading.users (
     wallet_address,
     starting_balance,
     current_balance,
     total_realized_pnl,
     created_at,
     total_trades,
     winning_trades,
     best_trade,
     worst_trade
) VALUES (?,?,?,?,?,?,?,?,?) IF NOT EXISTS";

const GET_USER_QUERY: &str = "SELECT wallet_address, starting_balance, current_balance, total_realized_pnl, created_at, total_trades, winning_trades, best_trade, worst_trade FROM paper_trading.users WHERE wallet_address = ?";

const UPDATE_USER_CURRENT_BALANCE_QUERY: &str =
    "UPDATE paper_trading.users SET current_balance = ? WHERE wallet_address = ?";

const UPDATE_USER_PNL_QUERY: &str =
    "UPDATE paper_trading.users SET total_realized_pnl = ? WHERE wallet_address = ?";

pub(crate) const UPDATE_USER_QUERY: &str = "UPDATE paper_trading.users SET current_balance = ?, total_realized_pnl = ?, total_trades = ?, winning_trades = ?, best_trade = ?, worst_trade = ? WHERE wallet_address = ?";

#[derive(Clone)]
pub struct UserDb {
    create_user: PreparedStatement,
    get_user: PreparedStatement,
    update_balance: PreparedStatement,
    update_pnl: PreparedStatement,
    pub(crate) update_user: PreparedStatement,
}

impl UserDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            create_user: session.prepare(CREATE_USER_QUERY).await?,
            get_user: session.prepare(GET_USER_QUERY).await?,
            update_balance: session.prepare(UPDATE_USER_CURRENT_BALANCE_QUERY).await?,
            update_pnl: session.prepare(UPDATE_USER_PNL_QUERY).await?,
            update_user: session.prepare(UPDATE_USER_QUERY).await?,
        })
    }

    pub async fn create_user(&self, session: &Session, user: Users) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.create_user,
                (
                    &user.wallet_address,
                    user.starting_balance,
                    user.current_balance,
                    user.total_realized_pnl,
                    user.created_at,
                    user.total_trades,
                    user.winning_trades,
                    user.best_trade,
                    user.worst_trade,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn get_user(
        &self,
        session: &Session,
        wallet_address: &str,
    ) -> Result<Option<Users>, Box<dyn Error>> {
        let result = session
            .execute_unpaged(&self.get_user, (wallet_address,))
            .await?
            .into_rows_result()?;

        let user = result
            .rows::<(String, f64, f64, f64, i64, i32, i32, f64, f64)>()?
            .filter_map(|r| r.ok())
            .map(
                |(
                    wallet_address,
                    starting_balance,
                    current_balance,
                    total_realized_pnl,
                    created_at,
                    total_trades,
                    winning_trades,
                    best_trade,
                    worst_trade,
                )| Users {
                    wallet_address,
                    starting_balance,
                    current_balance,
                    total_realized_pnl,
                    created_at,
                    total_trades,
                    winning_trades,
                    best_trade,
                    worst_trade,
                },
            )
            .next();

        Ok(user)
    }

    pub async fn update_balance(
        &self,
        session: &Session,
        wallet_address: &str,
        balance: f64,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(&self.update_balance, (balance, wallet_address))
            .await?;
        Ok(())
    }

    pub async fn update_pnl(
        &self,
        session: &Session,
        wallet_address: &str,
        pnl: f64,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(&self.update_pnl, (pnl, wallet_address))
            .await?;
        Ok(())
    }

    pub async fn update_all(
        &self,
        session: &Session,
        wallet_address: &str,
        balance: f64,
        total_realized_pnl: f64,
        total_trades: i32,
        winning_trades: i32,
        best_trade: f64,
        worst_trade: f64,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.update_user,
                (
                    balance,
                    total_realized_pnl,
                    total_trades,
                    winning_trades,
                    best_trade,
                    worst_trade,
                    wallet_address,
                ),
            )
            .await?;
        Ok(())
    }
}
