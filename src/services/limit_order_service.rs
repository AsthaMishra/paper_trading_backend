use std::{error::Error, sync::Arc};

use scylla::client::session::Session;
use uuid::Uuid;

use crate::{LimitOrder, LimitOrdersDb};

#[derive(Clone)]
pub struct LimitOrderService {
    db: Arc<Session>,
    limit_orders_db: LimitOrdersDb,
}

impl LimitOrderService {
    pub async fn new(db: Arc<Session>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            limit_orders_db: LimitOrdersDb::new(&db).await?,
            db,
        })
    }

    pub async fn create(
        &self,
        wallet_address: String,
        asset: String,
        side: String,
        order_type: String,
        quantity: f64,
        limit_price: f64,
    ) -> Result<LimitOrder, Box<dyn Error>> {
        let order = LimitOrder {
            wallet_address,
            id: Uuid::new_v4(),
            asset,
            side,
            order_type,
            quantity,
            limit_price,
            created_at: chrono::Utc::now().timestamp_millis(),
        };
        self.limit_orders_db.insert(&self.db, &order).await?;
        Ok(order)
    }

    pub async fn cancel(
        &self,
        wallet_address: &str,
        id: Uuid,
    ) -> Result<(), Box<dyn Error>> {
        self.limit_orders_db.delete(&self.db, wallet_address, id).await
    }

    pub async fn get_orders(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<LimitOrder>, Box<dyn Error>> {
        self.limit_orders_db.get_by_wallet(&self.db, wallet_address).await
    }
}
