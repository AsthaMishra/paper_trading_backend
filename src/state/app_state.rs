use std::sync::Arc;

use scylla::client::session::Session;

pub struct AppState{
    pub db: Arc<Session>
}

impl AppState{
    pub fn new (db: Arc<Session>) -> Self{
        Self { db }
    }
}