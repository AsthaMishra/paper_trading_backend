use std::{error::Error, sync::Arc};

use base64::{Engine, prelude::BASE64_STANDARD};
use scylla::{client::session::Session, response::PagingState};

use crate::{PortfolioPerformance, PortfolioPerformanceDB};


#[derive(Clone)]
pub struct PortfolioPerformanceService {
     db: Arc<Session>,
     portfolio_performance: PortfolioPerformanceDB,
}

impl PortfolioPerformanceService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            portfolio_performance: PortfolioPerformanceDB::new(&db).await?,
            db,
        })
    }

    pub async fn get_performance_portfolio(
        &self,
        wallet_address: String,
        page_size: i32,
        page_token: Option<String>,
    ) -> Result<(Vec<PortfolioPerformance>, Option<String>), Box<dyn Error>> {
        let paging_state = match page_token {
            Some(input) => {
                let bytes = BASE64_STANDARD.decode(input)?;
                PagingState::new_from_raw_bytes(bytes)
            }
            None => PagingState::start(),
        };

        let (history, next_paging_state) = self
            .portfolio_performance
            .get_portforlio(&self.db, page_size, paging_state, wallet_address)
            .await?;

        let next_state = next_paging_state.and_then(|ps| {
            ps.as_bytes_slice()
                .map(|s| BASE64_STANDARD.encode(s.as_ref()))
        });

        Ok((history, next_state))
    }
}
