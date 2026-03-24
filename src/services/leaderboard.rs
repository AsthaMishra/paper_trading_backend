use std::{error::Error, sync::Arc};

use scylla::client::session::Session;

use crate::{Leaderboard, LeaderboardDb};

#[derive(Clone)]
pub struct LeaderboardService {
    db: Arc<Session>,
    leaderboard_db: LeaderboardDb,
}

impl LeaderboardService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            leaderboard_db: LeaderboardDb::new(&db).await?,
            db,
        })
    }

    pub async fn get_top(
        &self,
        bucket: &str,
        limit: i32,
    ) -> Result<Vec<Leaderboard>, Box<dyn Error>> {
        self.leaderboard_db.get_top(&self.db, bucket, limit).await
    }
}
