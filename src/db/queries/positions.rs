use scylla::{client::session::Session, statement::prepared::PreparedStatement};
use std::error::Error;

const CREATE_POSITION: &str = "INSERT INTO paper_trading.positions(
    wallet_address,
    asset,
    quantity,
    avg_entry_price,
    realized_pnl,
    opened_at,
    updated_at
) VALUES (?,?,?,?,?,?,?)
";

const UPDATE_POSITION: &str = "UPDATE paper_trading.positions
SET
    quantity = ?,
    avg_entry_price = ?,
    updated_at = ?
    WHERE wallet_address = ? AND asset = ?
";

const PARTIAL_SELL: &str = "UPDATE paper_trading.positions
SET
    quantity = ?,
    realized_pnl = ?,
    updated_at = ?
    WHERE wallet_address = ? AND asset = ?
";

const FULL_SELL: &str = "DELETE from paper_trading.positions
WHERE wallet_address = ? AND asset = ?";

pub struct PositionsDb {
    create_position: PreparedStatement,
    update_position: PreparedStatement,
    partial_sell: PreparedStatement,
    full_sell: PreparedStatement,
}

impl PositionsDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            create_position: session.prepare(CREATE_POSITION).await?,
            update_position: session.prepare(UPDATE_POSITION).await?,
            partial_sell: session.prepare(PARTIAL_SELL).await?,
            full_sell: session.prepare(FULL_SELL).await?,
        })
    }

    pub async fn create_position(
        &self,
        session: &Session,
        wallet_address: &String,
        asset: &String,
        quantity: f64,
        avg_entry_price: f64,
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
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn update_position(
        &self,
        session: &Session,
        wallet_address: &String,
        asset: &String,
        quantity: f64,
        avg_entry_price: f64,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.update_position,
                (
                    quantity,
                    avg_entry_price,
                    chrono::Utc::now().timestamp_millis(),
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
        wallet_address: &String,
        asset: &String,
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

    pub async fn full_sell(
        &self,
        session: &Session,
        wallet_address: &String,
        asset: &String,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(&self.full_sell, (wallet_address, asset))
            .await?;
        Ok(())
    }
}
