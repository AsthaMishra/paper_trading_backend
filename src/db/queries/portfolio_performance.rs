use std::error::Error;

use scylla::{
    client::session::Session,
    response::{PagingState, PagingStateResponse},
    statement::prepared::PreparedStatement,
};

use crate::PortfolioPerformance;

const SNAPSHOT: &str = "INSERT INTO paper_trading.portfolio_performance ( wallet_address ,
    timestamp ,
    balance ,
    realized_pnl ) VALUES (?, ?, ?,?)";

const GET_PORTFOLIO: &str = "SELECT wallet_address ,
    timestamp ,
    balance ,
    realized_pnl FROM paper_trading.portfolio_performance WHERE wallet_address = ?";

#[derive(Clone)]
pub struct PortfolioPerformanceDB {
    snapshot_statement: PreparedStatement,
    get_portfolio_statement: PreparedStatement,
}

impl PortfolioPerformanceDB {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            snapshot_statement: session.prepare(SNAPSHOT).await?,
            get_portfolio_statement: session.prepare(GET_PORTFOLIO).await?,
        })
    }

    pub async fn snapshot(
        &self,
        session: &Session,
        wallet_address: String,
        balance: f64,
        realized_pnl: f64,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.snapshot_statement,
                (
                    wallet_address,
                    chrono::Utc::now().timestamp_millis(),
                    balance,
                    realized_pnl,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn get_portforlio(
        &self,
        session: &Session,
        page_size: i32,
        paging_state: PagingState,
        wallet_address: String,
    ) -> Result<(Vec<PortfolioPerformance>, Option<PagingState>), Box<dyn Error>> {
        let mut statement = self.get_portfolio_statement.clone();
        statement.set_page_size(page_size);

        let (response, paging_state_response) = session
            .execute_single_page(
                &statement,
                (wallet_address,),
                paging_state,
            )
            .await?;

        let portfolio: Vec<PortfolioPerformance> = response
            .into_rows_result()?
            .rows::<(String, i64, f64, f64)>()?
            .filter_map(|r| r.ok())
            .map(
                |(wallet_address, timestamp, balance, realized_pnl)| PortfolioPerformance {
                    wallet_address,
                    timestamp,
                    balance,
                    realized_pnl,
                },
            )
            .collect();

        let next_paging_state = match paging_state_response {
            PagingStateResponse::HasMorePages { state } => Some(state),
            PagingStateResponse::NoMorePages => None,
        };

        Ok((portfolio, next_paging_state))
    }
}
