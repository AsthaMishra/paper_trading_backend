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

// ── Test 2: 50 concurrent buy trades ────────────────────────────────────────

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
                    "quantity": 1.0,
                    "order_price": 100.0
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

// ── Test 3: 50 concurrent portfolio reads ───────────────────────────────────

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
                "quantity": 1.0,
                "order_price": 100.0
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
            (i, res.status().as_u16())
        });
    }

    let mut success = 0;
    let mut failed = 0;
    while let Some(result) = set.join_next().await {
        let (i, status) = result.unwrap();
        if status == 200 {
            success += 1;
        } else {
            failed += 1;
            println!("  [portfolio {}] failed with status {}", i, status);
        }
    }

    println!(
        "\nPortfolio reads — {}/{} succeeded in {:?}",
        success, CONCURRENT_REQUESTS, start.elapsed()
    );
    assert_eq!(failed, 0, "{} portfolio reads failed", failed);
}

// ── Test 4: Mixed load — create + trade + read simultaneously ────────────────

#[tokio::test]
async fn test_mixed_concurrent_load() {
    let client = Client::new();

    // Pre-create users for read/trade tasks
    for i in 0..CONCURRENT_REQUESTS {
        create_user(&client, &wallet(i, "mixed_user"))
            .await
            .expect("setup failed");
    }

    let mut set = JoinSet::new();
    let start = Instant::now();

    for i in 0..CONCURRENT_REQUESTS {
        let client = client.clone();
        // Rotate through 3 different request types
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
                // Buy trade
                let w = wallet(i, "mixed_user");
                set.spawn(async move {
                    let res = client
                        .post(format!("{}/trade", BASE_URL))
                        .json(&json!({
                            "wallet_address": w,
                            "asset": "SOL",
                            "side": "buy",
                            "quantity": 0.5,
                            "order_price": 100.0
                        }))
                        .send()
                        .await
                        .expect("request failed");
                    (i, "buy_trade", res.status().as_u16())
                });
            }
            _ => {
                // Portfolio read
                let w = wallet(i, "mixed_user");
                set.spawn(async move {
                    let res = client
                        .get(format!("{}/portfolio/{}", BASE_URL, w))
                        .send()
                        .await
                        .expect("request failed");
                    (i, "portfolio_read", res.status().as_u16())
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
