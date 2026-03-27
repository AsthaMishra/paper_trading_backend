use std::{collections::{HashMap, HashSet}, time::Duration};

use paper_trading_backend::{LimitOrder, LimitOrderService, MarketDataService, TradeService};

pub async fn run(
    trade_service: TradeService,
    limit_order_service: LimitOrderService,
    market_data: MarketDataService,
) {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        if let Err(e) = 
        tick(&trade_service, &limit_order_service, &market_data).await {
            log::error!("background worker error: {}", e);
        }
    }
}

async fn tick(
    trade_service: &TradeService,
    limit_order_service: &LimitOrderService,
    market_data: &MarketDataService,
) -> Result<(), Box<dyn std::error::Error>> {
    let orders: Vec<LimitOrder> = limit_order_service.get_all().await?;
    if orders.is_empty() {
        return Ok(());
    }

    // Collect unique assets across all pending orders
    let assets: Vec<String> = orders
        .iter()
        .map(|o| o.asset.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let prices: HashMap<String, f64> = market_data.get_prices(&assets).await
    .map_err(|e| format!("Exception in getting market prices {}", e))?;

    let mut executed: HashSet<(String, String)> = HashSet::new();

    for order in orders {
        if executed.contains(&(order.wallet_address.clone(), order.asset.clone())) {
            continue;
        }

        let current_price = match prices.get(&order.asset) {
            Some(p) => *p,
            None => {
                log::warn!("no price found for asset {}", order.asset);
                continue;
            }
        };

        let triggered = match order.order_type.as_str() {
            "buy_limit" => current_price <= order.limit_price,
            "stop_loss" => current_price <= order.limit_price,
            "take_profit" => current_price >= order.limit_price,
            _ => false,
        };

        if !triggered {
            continue;
        }

        let result: Result<(), String> = match order.side.as_str() {
            "buy" => {
                trade_service
                    .buy(
                        order.wallet_address.clone(),
                        order.asset.clone(),
                        order.quantity,
                        current_price,
                        None,
                        None,
                    )
                    .await
                    .map_err(|e| e.to_string())
            }
            "sell" => {
                trade_service
                    .sell(
                        order.wallet_address.clone(),
                        order.asset.clone(),
                        order.quantity,
                        current_price,
                    )
                    .await
                    .map_err(|e| e.to_string())
            }
            _ => continue,
        };

        match result {
            Ok(_) => {
                log::info!(
                    "executed {} order for {} {} at {}",
                    order.order_type, order.quantity, order.asset, current_price
                );
                executed.insert((order.wallet_address.clone(), order.asset.clone()));
                if let Err(e) = limit_order_service
                    .cancel_by_wallet_and_asset(&order.wallet_address, &order.asset)
                    .await
                {
                    log::error!("failed to clean up paired orders: {}", e);
                }
            }
            Err(e) => {
                log::error!(
                    "failed to execute {} order {}: {}",
                    order.order_type, order.id, e
                );
            }
        }
    }

    Ok(())
}
