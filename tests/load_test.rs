use std::time::Instant;

use reqwest::Client;
use serde_json::{Value, json};
use tokio::task::JoinSet;

const BASE_URL: &str = "http://localhost:8080";
const CONCURRENT_REQUESTS: usize = 50;

fn wallet(i: usize, prefix: &str) -> String {
    format!("{}_{}", prefix, i)
}

async fn create_user(client: &Client, wallet_address: &str) -> Result<(), String> {
    let res = client
        .post(format!("{}/users", BASE_URL))
        .json(&json!({ "wallet_address": wallet_address }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("create_user failed: {}", res.status()))
    }
}

// ── Test 1: 50 concurrent user creations ────────────────────────────────────

#[tokio::test]
async fn test_concurrent_user_creation() {
    let client = Client::new();
    let mut set = JoinSet::new();
    let start = Instant::now();

    for i in 0..CONCURRENT_REQUESTS {
        let client = client.clone();
        let wallet = wallet(i, "load_user");
        set.spawn(async move {
            let res = client
                .post(format!("{}/users", BASE_URL))
                .json(&json!({ "wallet_address": wallet }))
                .send()
                .await
                .expect("request failed");
            (i, res.status().as_u16())
        });
    }

    let mut success = 0;
    let mut failed = 0;
    while let Some(result) = set.join_next().await {
        let (i, status) = result.unwrap();
        if status == 200 || status == 201 {
            success += 1;
        } else {
            failed += 1;
            println!("  [user {}] failed with status {}", i, status);
        }
    }

    println!(
        "\nUser creation — {}/{} succeeded in {:?}",
        success, CONCURRENT_REQUESTS, start.elapsed()
    );
    assert_eq!(failed, 0, "{} user creations failed", failed);
}

// ── Test 2: prices endpoint returns live data ────────────────────────────────

#[tokio::test]
async fn test_prices_endpoint() {
    let client = Client::new();
    let start = Instant::now();

    let res = client
        .get(format!("{}/prices?tokens=SOL,ETH,BTC", BASE_URL))
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status().as_u16(), 200, "prices endpoint failed");

    let body: Value = res.json().await.expect("invalid json");
    assert!(body.get("SOL").is_some(), "SOL price missing from response");
    assert!(body.get("ETH").is_some(), "ETH price missing from response");
    assert!(body.get("BTC").is_some(), "BTC price missing from response");

    let sol_price = body["SOL"].as_f64().unwrap();
    assert!(sol_price > 0.0, "SOL price must be > 0, got {}", sol_price);

    println!(
        "\nPrices — SOL=${:.2} ETH=${:.2} BTC=${:.2} fetched in {:?}",
        sol_price,
        body["ETH"].as_f64().unwrap_or(0.0),
        body["BTC"].as_f64().unwrap_or(0.0),
        start.elapsed()
    );
}

// ── Test 3: 50 concurrent buy trades (price fetched server-side) ─────────────

#[tokio::test]
async fn test_concurrent_buy_trades() {
    let client = Client::new();

    // Create users first (sequential setup)
    for i in 0..CONCURRENT_REQUESTS {
        create_user(&client, &wallet(i, "trade_user"))
            .await
            .expect("setup: user creation failed");
    }

    let mut set = JoinSet::new();
    let start = Instant::now();

    for i in 0..CONCURRENT_REQUESTS {
        let client = client.clone();
        let wallet = wallet(i, "trade_user");
        set.spawn(async move {
            let res = client
                .post(format!("{}/trade", BASE_URL))
                .json(&json!({
                    "wallet_address": wallet,
                    "asset": "SOL",
                    "side": "buy",
                    "quantity": 1.0
                    // no order_price — server fetches live price from Jupiter
                }))
                .send()
                .await
                .expect("request failed");
            let status = res.status().as_u16();
            let body: Value = res.json().await.unwrap_or(Value::Null);
            (i, status, body)
        });
    }

    let mut success = 0;
    let mut failed = 0;
    while let Some(result) = set.join_next().await {
        let (i, status, body) = result.unwrap();
        if status == 200 || status == 201 {
            success += 1;
        } else {
            failed += 1;
            println!("  [trade {}] status {}: {}", i, status, body);
        }
    }

    println!(
        "\nBuy trades — {}/{} succeeded in {:?}",
        success, CONCURRENT_REQUESTS, start.elapsed()
    );
    assert_eq!(failed, 0, "{} buy trades failed", failed);
}

// ── Test 4: portfolio includes unrealized PnL and live price ─────────────────

#[tokio::test]
async fn test_portfolio_with_unrealized_pnl() {
    let client = Client::new();
    let wallet_addr = "pnl_test_wallet";

    create_user(&client, wallet_addr).await.expect("setup failed");

    client
        .post(format!("{}/trade", BASE_URL))
        .json(&json!({
            "wallet_address": wallet_addr,
            "asset": "SOL",
            "side": "buy",
            "quantity": 2.0
        }))
        .send()
        .await
        .expect("buy failed");

    let res = client
        .get(format!("{}/portfolio/{}", BASE_URL, wallet_addr))
        .send()
        .await
        .expect("portfolio request failed");

    assert_eq!(res.status().as_u16(), 200);

    let positions: Vec<Value> = res.json().await.expect("invalid json");
    assert!(!positions.is_empty(), "expected at least one open position");

    let pos = &positions[0];
    assert_eq!(pos["asset"].as_str().unwrap(), "SOL");

    // Verify enriched fields are present
    let current_price = pos["current_price"].as_f64()
        .expect("current_price must be present");
    let unrealized_pnl = pos["unrealized_pnl"].as_f64()
        .expect("unrealized_pnl must be present");
    let avg_entry = pos["avg_entry_price"].as_f64()
        .expect("avg_entry_price must be present");
    let quantity = pos["quantity"].as_f64().unwrap();

    let expected_pnl = (current_price - avg_entry) * quantity;
    assert!(
        (unrealized_pnl - expected_pnl).abs() < 0.01,
        "unrealized_pnl={} does not match expected ({} - {}) * {} = {}",
        unrealized_pnl, current_price, avg_entry, quantity, expected_pnl
    );

    println!(
        "\nPortfolio PnL — SOL position: qty={} avg_entry=${:.2} current=${:.2} unrealized_pnl=${:.2}",
        quantity, avg_entry, current_price, unrealized_pnl
    );
}

// ── Test 5: 50 concurrent portfolio reads ────────────────────────────────────

#[tokio::test]
async fn test_concurrent_portfolio_reads() {
    let client = Client::new();

    // Setup: create users and buy positions
    for i in 0..CONCURRENT_REQUESTS {
        let w = wallet(i, "portfolio_user");
        create_user(&client, &w).await.expect("setup failed");
        client
            .post(format!("{}/trade", BASE_URL))
            .json(&json!({
                "wallet_address": w,
                "asset": "SOL",
                "side": "buy",
                "quantity": 1.0
            }))
            .send()
            .await
            .expect("setup trade failed");
    }

    let mut set = JoinSet::new();
    let start = Instant::now();

    for i in 0..CONCURRENT_REQUESTS {
        let client = client.clone();
        let wallet = wallet(i, "portfolio_user");
        set.spawn(async move {
            let res = client
                .get(format!("{}/portfolio/{}", BASE_URL, wallet))
                .send()
                .await
                .expect("request failed");
            let status = res.status().as_u16();
            let body: Vec<Value> = res.json().await.unwrap_or_default();
            // Verify response shape
            let has_unrealized_pnl = body.first()
                .map(|p| p.get("unrealized_pnl").is_some())
                .unwrap_or(true); // empty portfolio is also valid
            (i, status, has_unrealized_pnl)
        });
    }

    let mut success = 0;
    let mut failed = 0;
    while let Some(result) = set.join_next().await {
        let (i, status, has_unrealized_pnl) = result.unwrap();
        if status == 200 && has_unrealized_pnl {
            success += 1;
        } else {
            failed += 1;
            println!(
                "  [portfolio {}] status={} unrealized_pnl_present={}",
                i, status, has_unrealized_pnl
            );
        }
    }

    println!(
        "\nPortfolio reads — {}/{} succeeded in {:?}",
        success, CONCURRENT_REQUESTS, start.elapsed()
    );
    assert_eq!(failed, 0, "{} portfolio reads failed", failed);
}

// ── Test 6: Mixed load — create + trade + prices simultaneously ───────────────

#[tokio::test]
async fn test_mixed_concurrent_load() {
    let client = Client::new();

    // Pre-create users for trade tasks
    for i in 0..CONCURRENT_REQUESTS {
        create_user(&client, &wallet(i, "mixed_user"))
            .await
            .expect("setup failed");
    }

    let mut set = JoinSet::new();
    let start = Instant::now();

    for i in 0..CONCURRENT_REQUESTS {
        let client = client.clone();
        match i % 3 {
            0 => {
                // New user creation
                let w = wallet(i + 1000, "mixed_new");
                set.spawn(async move {
                    let res = client
                        .post(format!("{}/users", BASE_URL))
                        .json(&json!({ "wallet_address": w }))
                        .send()
                        .await
                        .expect("request failed");
                    (i, "create_user", res.status().as_u16())
                });
            }
            1 => {
                // Buy trade — no order_price, server fetches from Jupiter
                let w = wallet(i, "mixed_user");
                set.spawn(async move {
                    let res = client
                        .post(format!("{}/trade", BASE_URL))
                        .json(&json!({
                            "wallet_address": w,
                            "asset": "SOL",
                            "side": "buy",
                            "quantity": 0.5
                        }))
                        .send()
                        .await
                        .expect("request failed");
                    (i, "buy_trade", res.status().as_u16())
                });
            }
            _ => {
                // Live price fetch
                set.spawn(async move {
                    let res = client
                        .get(format!("{}/prices?tokens=SOL", BASE_URL))
                        .send()
                        .await
                        .expect("request failed");
                    (i, "price_fetch", res.status().as_u16())
                });
            }
        }
    }

    let mut success = 0;
    let mut failed = 0;
    while let Some(result) = set.join_next().await {
        let (i, op, status) = result.unwrap();
        if status == 200 || status == 201 {
            success += 1;
        } else {
            failed += 1;
            println!("  [{}:{}] failed with status {}", op, i, status);
        }
    }

    println!(
        "\nMixed load — {}/{} succeeded in {:?}",
        success, CONCURRENT_REQUESTS, start.elapsed()
    );
    assert_eq!(failed, 0, "{} mixed requests failed", failed);
}
