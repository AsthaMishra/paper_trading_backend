use scylla::{client::session::Session, statement::prepared::PreparedStatement};
use std::error::Error;

pub(crate) const CREATE_POSITION: &str = "INSERT INTO paper_trading.positions(
    wallet_address,
    asset,
    quantity,
    avg_entry_price,
    realized_pnl,
    opened_at,
    updated_at,
    stop_loss,
    take_profit
) VALUES (?,?,?,?,?,?,?,?,?)
";

pub(crate) const UPDATE_POSITION: &str = "UPDATE paper_trading.positions
SET
    quantity = ?,
    avg_entry_price = ?,
    updated_at = ?,  
    stop_loss = ?, 
    take_profit = ? 
    WHERE wallet_address = ? AND asset = ?
";

pub(crate) const SET_AUTO_SELL_CONDN: &str = "UPDATE paper_trading.positions 
SET 
    stop_loss = ?, take_profit = ? 
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

const GET_POSITION: &str = "SELECT wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at, stop_loss, take_profit FROM paper_trading.positions WHERE wallet_address = ? AND asset = ?";

const GET_ALL_POSITIONS: &str = "SELECT wallet_address, asset, quantity, avg_entry_price, realized_pnl, opened_at, updated_at, stop_loss, take_profit FROM paper_trading.positions WHERE wallet_address = ?";

#[derive(Clone)]
pub struct PositionsDb {
    pub(crate) create_position: PreparedStatement,
    pub(crate) update_position: PreparedStatement,
    pub(crate) partial_sell: PreparedStatement,
    pub(crate) full_sell: PreparedStatement,
    get_position: PreparedStatement,
    get_all_positions: PreparedStatement,
    set_auto_sell: PreparedStatement,
}

impl PositionsDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            create_position: session.prepare(CREATE_POSITION).await?,
            update_position: session.prepare(UPDATE_POSITION).await?,
            partial_sell: session.prepare(PARTIAL_SELL).await?,
            full_sell: session.prepare(FULL_SELL).await?,
            get_position: session.prepare(GET_POSITION).await?,
            get_all_positions: session.prepare(GET_ALL_POSITIONS).await?,
            set_auto_sell: session.prepare(SET_AUTO_SELL_CONDN).await?,
        })
    }

    pub async fn create_position(
        &self,
        session: &Session,
        wallet_address: &str,
        asset: &str,
        quantity: f64,
        avg_entry_price: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.create_position,
                (
                    wallet_address,
                    asset,
                    quantity,
                    avg_entry_price,
                    0.0,
                    chrono::Utc::now().timestamp_millis(),
                    chrono::Utc::now().timestamp_millis(),
                    stop_loss,
                    take_profit,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn update_position(
        &self,
        session: &Session,
        wallet_address: &str,
        asset: &str,
        quantity: f64,
        avg_entry_price: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.update_position,
                (
                    quantity,
                    avg_entry_price,
                    chrono::Utc::now().timestamp_millis(),
                    stop_loss,
                    take_profit,
                    wallet_address,
                    asset,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn partial_sell(
        &self,
        session: &Session,
        wallet_address: &str,
        asset: &str,
        quantity: f64,
        realized_pnl: f64,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.partial_sell,
                (
                    quantity,
                    realized_pnl,
                    chrono::Utc::now().timestamp_millis(),
                    wallet_address,
                    asset,
                ),
            )
            .await?;
        Ok(())
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
            .rows::<(
                String,
                String,
                f64,
                f64,
                f64,
                i64,
                i64,
                Option<f64>,
                Option<f64>,
            )>()?
            .filter_map(|r| r.ok())
            .map(
                |(
                    wallet_address,
                    asset,
                    quantity,
                    avg_entry_price,
                    realized_pnl,
                    opened_at,
                    updated_at,
                    stop_loss,
                    take_profit,
                )| {
                    crate::Positions {
                        wallet_address,
                        asset,
                        quantity,
                        avg_entry_price,
                        realized_pnl,
                        opened_at,
                        updated_at,
                        stop_loss,
                        take_profit,
                    }
                },
            )
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
            .rows::<(
                String,
                String,
                f64,
                f64,
                f64,
                i64,
                i64,
                Option<f64>,
                Option<f64>,
            )>()?
            .filter_map(|r| r.ok())
            .map(
                |(
                    wallet_address,
                    asset,
                    quantity,
                    avg_entry_price,
                    realized_pnl,
                    opened_at,
                    updated_at,
                    stop_loss,
                    take_profit,
                )| {
                    crate::Positions {
                        wallet_address,
                        asset,
                        quantity,
                        avg_entry_price,
                        realized_pnl,
                        opened_at,
                        updated_at,
                        stop_loss,
                        take_profit,
                    }
                },
            )
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

    pub async fn set_auto_sell(
        &self,
        session: &Session,
        wallet_address: &str,
        asset: &str,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.set_auto_sell,
                (stop_loss, take_profit, wallet_address, asset),
            )
            .await?;
        Ok(())
    }
}
