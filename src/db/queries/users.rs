use std::error::Error;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};

use crate::Users;

const CREATE_USER_QUERY: &str = "INSERT INTO paper_trading.users (
     wallet_address,
     starting_balance,
     current_balance,
     total_realized_pnl,
     created_at
) VALUES (?,?,?,?,?)";

const UPDATE_USER_CURRENT_BALANCE_QUERY: &str =
    "UPDATE paper_trading.users SET current_balance = ? WHERE wallet_address = ?";

const UPDATE_USER_PNL_QUERY: &str =
    "UPDATE paper_trading.users SET total_realized_pnl = ? WHERE wallet_address = ?";

pub struct UserDb {
    create_user: PreparedStatement,
    update_balance: PreparedStatement,
    update_pnl: PreparedStatement,
}

impl UserDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            create_user: session.prepare(CREATE_USER_QUERY).await?,
            update_balance: session.prepare(UPDATE_USER_CURRENT_BALANCE_QUERY).await?,
            update_pnl: session.prepare(UPDATE_USER_PNL_QUERY).await?,
        })
    }

    pub async fn create_user(&self, session: &Session, user: Users) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(&self.create_user, (
                &user.wallet_address,
                user.starting_balance,
                user.current_balance,
                user.total_realized_pnl,
                user.created_at,
            ))
            .await?;
        Ok(())
    }

    pub async fn update_balance(&self, session: &Session, wallet_address: &str, balance: f64) -> Result<(), Box<dyn Error>> {
        session.execute_unpaged(&self.update_balance, (balance, wallet_address)).await?;
        Ok(())
    }

    pub async fn update_pnl(&self, session: &Session, wallet_address: &str, pnl: f64) -> Result<(), Box<dyn Error>> {
        session.execute_unpaged(&self.update_pnl, (pnl, wallet_address)).await?;
        Ok(())
    }
}
