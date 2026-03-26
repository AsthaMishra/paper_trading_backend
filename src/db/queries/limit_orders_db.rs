use std::error::Error;

use scylla::{client::session::Session, statement::prepared::PreparedStatement};
use uuid::Uuid;

use crate::LimitOrder;

const INSERT: &str = "INSERT INTO paper_trading.limit_orders (
    wallet_address, id, asset, side, order_type, quantity, limit_price, created_at
) VALUES (?,?,?,?,?,?,?,?)";

const DELETE: &str = "DELETE FROM paper_trading.limit_orders
WHERE wallet_address = ? AND id = ?";

const GET_BY_WALLET: &str = "SELECT wallet_address, id, asset, side, order_type, quantity, limit_price, created_at
FROM paper_trading.limit_orders WHERE wallet_address = ?";

const GET_ALL: &str = "SELECT wallet_address, id, asset, side, order_type, quantity, limit_price, created_at
FROM paper_trading.limit_orders ALLOW FILTERING";

#[derive(Clone)]
pub struct LimitOrdersDb {
    insert: PreparedStatement,
    delete: PreparedStatement,
    get_by_wallet: PreparedStatement,
    get_all: PreparedStatement,
}

impl LimitOrdersDb {
    pub async fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            insert: session.prepare(INSERT).await?,
            delete: session.prepare(DELETE).await?,
            get_by_wallet: session.prepare(GET_BY_WALLET).await?,
            get_all: session.prepare(GET_ALL).await?,
        })
    }

    pub async fn insert(
        &self,
        session: &Session,
        order: &LimitOrder,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(
                &self.insert,
                (
                    &order.wallet_address,
                    order.id,
                    &order.asset,
                    &order.side,
                    &order.order_type,
                    order.quantity,
                    order.limit_price,
                    order.created_at,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn delete(
        &self,
        session: &Session,
        wallet_address: &str,
        id: Uuid,
    ) -> Result<(), Box<dyn Error>> {
        session
            .execute_unpaged(&self.delete, (wallet_address, id))
            .await?;
        Ok(())
    }

    pub async fn get_by_wallet(
        &self,
        session: &Session,
        wallet_address: &str,
    ) -> Result<Vec<LimitOrder>, Box<dyn Error>> {
        let result = session
            .execute_unpaged(&self.get_by_wallet, (wallet_address,))
            .await?
            .into_rows_result()?;

        let orders = result
            .rows::<(String, Uuid, String, String, String, f64, f64, i64)>()?
            .filter_map(|r| r.ok())
            .map(|(wallet_address, id, asset, side, order_type, quantity, limit_price, created_at)| {
                LimitOrder { wallet_address, id, asset, side, order_type, quantity, limit_price, created_at }
            })
            .collect();

        Ok(orders)
    }

    pub async fn get_all(
        &self,
        session: &Session,
    ) -> Result<Vec<LimitOrder>, Box<dyn Error>> {
        let result = session
            .execute_unpaged(&self.get_all, &[])
            .await?
            .into_rows_result()?;

        let orders = result
            .rows::<(String, Uuid, String, String, String, f64, f64, i64)>()?
            .filter_map(|r| r.ok())
            .map(|(wallet_address, id, asset, side, order_type, quantity, limit_price, created_at)| {
                LimitOrder { wallet_address, id, asset, side, order_type, quantity, limit_price, created_at }
            })
            .collect();

        Ok(orders)
    }
}
