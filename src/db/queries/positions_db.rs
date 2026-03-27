use scylla::{client::session::Session, statement::prepared::PreparedStatement};
use std::error::Error;

pub(crate) const CREATE_POSITION: &str = "INSERT INTO paper_trading.positions(
    wallet_address,
    asset,
    quantity,
    avg_entry_price,
    realized_pnl,
    opened_at,
    updated_at
) VALUES (?,?,?,?,?,?,?)
";

pub(crate) const UPDATE_POSITION: &str = "UPDATE paper_trading.positions
SET
    quantity = ?,
    avg_entry_price = ?,
    updated_at = ?
    WHERE wallet_address = ? AND asset = ?
";

pub(crate) const PARTIAL_SELL: &str = "UPDATE paper_trading.positions
SET
    quantity = ?,
    realized_pnl = ?,
    updated_at = ?
    WHERE wallet_address = ? AND asset = ?
";

pub(crate) const FULL_SELL: &str = "DELETE from paper_trading.positions
WHERE wallet_address = ? AND asset = ?";

const GET_POSITION: &str = "SELECT wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at FROM paper_trading.positions WHERE wallet_address = ? AND asset = ?";

const GET_ALL_POSITIONS: &str = "SELECT wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at FROM paper_trading.positions WHERE wallet_address = ?";

#[derive(Clone)]
pub struct PositionsDb {
    pub(crate) full_sell: PreparedStatement,
    get_position: PreparedStatement,
    get_all_positions: PreparedStatement,
}

impl PositionsDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            full_sell: session.prepare(FULL_SELL).await?,
            get_position: session.prepare(GET_POSITION).await?,
            get_all_positions: session.prepare(GET_ALL_POSITIONS).await?,
        })
    }

    pub async fn get_position(
        &self,
        session: &Session,
        wallet_address: &str,
        asset: &str,
    ) -> Result<Option<crate::Positions>, Box<dyn Error>> {
        let result = session
            .execute_unpaged(&self.get_position, (wallet_address, asset))
            .await?
            .into_rows_result()?;

        let position = result
            .rows::<(String, String, f64, f64, f64, i64, i64)>()?
            .filter_map(|r| r.ok())
            .map(|(wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at)| {
                crate::Positions { wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at }
            })
            .next();

        Ok(position)
    }

    pub async fn get_all_positions(
        &self,
        session: &Session,
        wallet_address: &str,
    ) -> Result<Vec<crate::Positions>, Box<dyn Error>> {
        let result = session
            .execute_unpaged(&self.get_all_positions, (wallet_address,))
            .await?
            .into_rows_result()?;

        let positions = result
            .rows::<(String, String, f64, f64, f64, i64, i64)>()?
            .filter_map(|r| r.ok())
            .map(|(wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at)| {
                crate::Positions { wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at }
            })
            .collect();

        Ok(positions)
    }

    pub async fn full_sell(
        &self,
        session: &Session,
        wallet_address: &str,
        asset: &str,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(&self.full_sell, (wallet_address, asset))
            .await?;
        Ok(())
    }
}
