use std::error::Error;

use scylla::{client::session::Session, statement::{batch::{Batch, BatchType}, prepared::PreparedStatement}};

use crate::Leaderboard;

const INSERT_LEADERBOARD_QUERY: &str =
    "INSERT INTO paper_trading.leaderboard (bucket, total_pnl, wallet_address) VALUES (?, ?, ?)";

const DELETE_LEADERBOARD_QUERY: &str =
    "DELETE FROM paper_trading.leaderboard WHERE bucket = ? AND total_pnl = ? AND wallet_address = ?";

const GET_LEADERBOARD_QUERY: &str = "SELECT bucket, total_pnl, wallet_address FROM paper_trading.leaderboard WHERE bucket = ? LIMIT ?";

#[derive(Clone)]
pub struct LeaderboardDb {
    upsert_batch: Batch,
    get: PreparedStatement,
}

impl LeaderboardDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        let delete = session.prepare(DELETE_LEADERBOARD_QUERY).await?;
        let insert = session.prepare(INSERT_LEADERBOARD_QUERY).await?;

        let mut upsert_batch = Batch::new(BatchType::Unlogged);
        upsert_batch.append_statement(delete);
        upsert_batch.append_statement(insert);

        Ok(Self {
            upsert_batch,
            get: session.prepare(GET_LEADERBOARD_QUERY).await?,
        })
    }

    pub async fn upsert(
        &self,
        session: &Session,
        old_pnl: f64,
        entry: Leaderboard,
    ) -> Result<(), Box<dyn Error>> {
        session.batch(&self.upsert_batch, (
            (&entry.bucket, old_pnl, &entry.wallet_address),
            (&entry.bucket, entry.total_pnl, &entry.wallet_address),
        )).await?;
        Ok(())
    }

    pub async fn get_top(
        &self,
        session: &Session,
        bucket: &str,
        limit: i32,
    ) -> Result<Vec<Leaderboard>, Box<dyn Error>> {
        let result = session
            .execute_unpaged(&self.get, (bucket, limit))
            .await?
            .into_rows_result()?;

        let entries = result
            .rows::<(String, f64, String)>()?
            .filter_map(|r| r.ok())
            .map(|(bucket, total_pnl, wallet_address)| Leaderboard {
                bucket,
                total_pnl,
                wallet_address,
            })
            .collect();

        Ok(entries)
    }
}
