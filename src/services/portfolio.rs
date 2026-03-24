use std::{error::Error, sync::Arc};

use scylla::client::session::Session;

use crate::{Positions, PositionsDb};

#[derive(Clone)]
pub struct PortfolioService {
    db: Arc<Session>,
    positions_db: PositionsDb,
}

impl PortfolioService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            positions_db: PositionsDb::new(&db).await?,
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
}
