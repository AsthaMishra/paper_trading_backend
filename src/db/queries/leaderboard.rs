use std::error::Error;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};

use crate::Leaderboard;

const UPSERT_LEADERBOARD_QUERY: &str =
    "INSERT INTO paper_trading.leaderboard (bucket, total_pnl, wallet_address) VALUES (?, ?, ?)";

const GET_LEADERBOARD_QUERY: &str =
    "SELECT bucket, total_pnl, wallet_address FROM paper_trading.leaderboard WHERE bucket = ? LIMIT ?";

pub struct LeaderboardDb {
    upsert: PreparedStatement,
    get: PreparedStatement,
}

impl LeaderboardDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            upsert: session.prepare(UPSERT_LEADERBOARD_QUERY).await?,
            get: session.prepare(GET_LEADERBOARD_QUERY).await?,
        })
    }

    pub async fn upsert(
        &self,
        session: &Session,
        entry: Leaderboard,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(&self.upsert, (&entry.bucket, entry.total_pnl, &entry.wallet_address))
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
