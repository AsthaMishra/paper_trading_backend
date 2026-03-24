use std::{error::Error, sync::Arc};

use scylla::client::session::Session;

use crate::{UserDb, Users};

#[derive(Clone)]
pub struct UserService {
    db: Arc<Session>,
    user_db: UserDb,
}

const STARTING_BALANCE: f64 = 10000.0;

impl UserService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            user_db: UserDb::new(&db).await?,
            db,
        })
    }

    pub async fn create_user(&self, wallet_address: String) -> Result<(), Box<dyn Error>> {
        let user = Users {
            wallet_address,
            starting_balance: STARTING_BALANCE,
            current_balance: STARTING_BALANCE,
            total_realized_pnl: 0.0,
            created_at: chrono::Utc::now().timestamp_millis(),
        };
        self.user_db.create_user(&self.db, user).await
    }

    pub async fn get_user(&self, wallet_address: &str) -> Result<Option<Users>, Box<dyn Error>> {
        self.user_db.get_user(&self.db, wallet_address).await
    }
}
