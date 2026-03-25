use std::error::Error;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};

use crate::Leaderboard;

const INSERT_LEADERBOARD_QUERY: &str =
    "INSERT INTO paper_trading.leaderboard (bucket, total_pnl, wallet_address) VALUES (?, ?, ?)";

const DELETE_LEADERBOARD_QUERY: &str =
    "DELETE FROM paper_trading.leaderboard WHERE bucket = ? AND total_pnl = ? AND wallet_address = ?";

const GET_LEADERBOARD_QUERY: &str = "SELECT bucket, total_pnl, wallet_address FROM paper_trading.leaderboard WHERE bucket = ? LIMIT ?";

#[derive(Clone)]
pub struct LeaderboardDb {
    insert: PreparedStatement,
    delete: PreparedStatement,
    get: PreparedStatement,
}

impl LeaderboardDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            insert: session.prepare(INSERT_LEADERBOARD_QUERY).await?,
            delete: session.prepare(DELETE_LEADERBOARD_QUERY).await?,
            get: session.prepare(GET_LEADERBOARD_QUERY).await?,
        })
    }

    pub async fn upsert(
        &self,
        session: &Session,
        old_pnl: f64,
        entry: Leaderboard,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.delete,
                (&entry.bucket, old_pnl, &entry.wallet_address),
            )
            .await?;
        session
            .execute_unpaged(
                &self.insert,
                (&entry.bucket, entry.total_pnl, &entry.wallet_address),
            )
            .await?;
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
