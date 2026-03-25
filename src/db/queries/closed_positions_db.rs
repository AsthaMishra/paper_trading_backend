use std::error::Error;

use scylla::{
    client::session::Session,
    response::{PagingState, PagingStateResponse},
    statement::prepared::PreparedStatement,
};

use crate::ClosedPosition;

pub(crate) const INSERT_CLOSED_POSITION: &str = "INSERT INTO paper_trading.closed_positions (
    wallet_address,
    closed_at,
    asset,
    opened_at,
    quantity,
    avg_entry_price,
    exit_price,
    realized_pnl
) VALUES (?,?,?,?,?,?,?,?)";

const GET_CLOSED_POSITIONS: &str = "SELECT wallet_address, closed_at, asset, opened_at, quantity, avg_entry_price, exit_price, realized_pnl
    FROM paper_trading.closed_positions WHERE wallet_address = ?";

#[derive(Clone)]
pub struct ClosedPositionsDb {
    pub(crate) insert: PreparedStatement,
    get: PreparedStatement,
}

impl ClosedPositionsDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            insert: session.prepare(INSERT_CLOSED_POSITION).await?,
            get: session.prepare(GET_CLOSED_POSITIONS).await?,
        })
    }

    pub async fn insert(
        &self,
        session: &Session,
        position: &ClosedPosition,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.insert,
                (
                    &position.wallet_address,
                    position.closed_at,
                    &position.asset,
                    position.opened_at,
                    position.quantity,
                    position.avg_entry_price,
                    position.exit_price,
                    position.realized_pnl,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn get_closed_positions(
        &self,
        session: &Session,
        wallet_address: &str,
        page_size: i32,
        paging_state: PagingState,
    ) -> Result<(Vec<ClosedPosition>, Option<PagingState>), Box<dyn Error>> {
        let mut statement = self.get.clone();
        statement.set_page_size(page_size);

        let (result, paging_response) = session
            .execute_single_page(&statement, (wallet_address,), paging_state)
            .await?;

        let positions = result
            .into_rows_result()?
            .rows::<(String, i64, String, i64, f64, f64, f64, f64)>()?
            .filter_map(|r| r.ok())
            .map(
                |(wallet_address, closed_at, asset, opened_at, quantity, avg_entry_price, exit_price, realized_pnl)| {
                    ClosedPosition { wallet_address, closed_at, asset, opened_at, quantity, avg_entry_price, exit_price, realized_pnl }
                },
            )
            .collect();

        let next_page = match paging_response {
            PagingStateResponse::HasMorePages { state } => Some(state),
            PagingStateResponse::NoMorePages => None,
        };

        Ok((positions, next_page))
    }
}
