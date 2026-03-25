use std::{error::Error, sync::Arc};

use scylla::{client::session::Session, response::PagingState};

use crate::{ClosedPosition, ClosedPositionsDb, Positions, PositionsDb};

#[derive(Clone)]
pub struct PortfolioService {
    db: Arc<Session>,
    positions_db: PositionsDb,
    closed_positions_db: ClosedPositionsDb,
}

impl PortfolioService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            positions_db: PositionsDb::new(&db).await?,
            closed_positions_db: ClosedPositionsDb::new(&db).await?,
            db,
        })
    }

    pub async fn get_positions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<Positions>, Box<dyn Error>> {
        self.positions_db
            .get_all_positions(&self.db, wallet_address)
            .await
    }

    pub async fn set_auto_sell(
        &self,
        wallet_address: &str,
        asset: &str,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<(), Box<dyn Error>> {
        let pos = self
            .positions_db
            .get_position(&self.db, wallet_address, asset)
            .await?;
        if pos.is_none() {
            return Err("Position not found".into());
        }
        self.positions_db
            .set_auto_sell(&self.db, wallet_address, asset, stop_loss, take_profit)
            .await
    }

    pub async fn get_closed_positions(
        &self,
        wallet_address: &str,
        page_size: i32,
        paging_state: PagingState,
    ) -> Result<(Vec<ClosedPosition>, Option<PagingState>), Box<dyn Error>> {
        self.closed_positions_db
            .get_closed_positions(&self.db, wallet_address, page_size, paging_state)
            .await
    }
}
