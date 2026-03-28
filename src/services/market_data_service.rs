use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};

const BINANCE_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/price";

#[derive(Deserialize)]
struct BinanceTicker {
    symbol: String,
    price: String,
}

#[derive(Clone)]
pub struct MarketDataService {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, (f64, Instant)>>>,
    cache_ttl: Duration,
    // Assets the worker should refresh every tick.
    // Populated when a trade or portfolio read occurs for an asset.
    watched: Arc<RwLock<HashSet<String>>>,
    // Serializes concurrent Binance fetches (double-checked locking).
    fetch_lock: Arc<Mutex<()>>,
}

impl MarketDataService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(5),
            watched: Arc::new(RwLock::new(HashSet::new())),
            fetch_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Register an asset to be kept warm by the background worker.
    pub async fn watch(&self, token: &str) {
        let mut watched = self.watched.write().await;
        watched.insert(token.to_string());
    }

    /// Watch and immediately fetch prices for the given tokens.
    /// Call this on startup to pre-warm the cache before serving requests.
    pub async fn warm_up(&self, tokens: &[&str]) {
        let owned: Vec<String> = tokens.iter().map(|s| s.to_string()).collect();
        {
            let mut watched = self.watched.write().await;
            for t in &owned {
                watched.insert(t.clone());
            }
        }
        if let Err(e) = self.get_prices(&owned).await {
            log::warn!("warm_up failed: {}", e);
        }
    }

    /// All assets currently being watched. Used by the background worker.
    pub async fn watched_assets(&self) -> Vec<String> {
        self.watched.read().await.iter().cloned().collect()
    }

    /// Fetch and cache prices for all watched assets. Called by the worker every tick.
    pub async fn refresh_watched(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tokens = self.watched_assets().await;
        if tokens.is_empty() {
            return Ok(());
        }
        let result = self.fetch_from_binance(&tokens).await?;
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        for (token, price) in result {
            cache.insert(token, (price, now));
        }
        Ok(())
    }

    pub async fn get_prices(
        &self,
        tokens: &[String],
    ) -> Result<HashMap<String, f64>, Box<dyn std::error::Error + Send + Sync>> {
        if tokens.is_empty() {
            return Ok(HashMap::new());
        }

        // Fast path: all tokens already cached
        if let Some(result) = self.read_cache(tokens).await {
            return Ok(result);
        }

        // Serialize concurrent fetches — only one goroutine hits Binance.
        // All others wait, then the second cache check returns immediately.
        let _lock = self.fetch_lock.lock().await;
        if let Some(result) = self.read_cache(tokens).await {
            return Ok(result);
        }

        let result = self.fetch_from_binance(tokens).await?;
        {
            let mut cache = self.cache.write().await;
            let now = Instant::now();
            for (token, &price) in &result {
                cache.insert(token.clone(), (price, now));
            }
        }
        Ok(result)
    }

    pub async fn get_price(
        &self,
        token: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.get_prices(&[token.to_string()]).await?;
        prices
            .get(token)
            .copied()
            .ok_or_else(|| format!("price not found for {}", token).into())
    }

    async fn read_cache(&self, tokens: &[String]) -> Option<HashMap<String, f64>> {
        let cache = self.cache.read().await;
        let now = Instant::now();
        let all_fresh = tokens.iter().all(|t| {
            cache
                .get(t)
                .map(|(_, ts)| now.duration_since(*ts) < self.cache_ttl)
                .unwrap_or(false)
        });
        if all_fresh {
            Some(
                tokens
                    .iter()
                    .filter_map(|t| cache.get(t).map(|(price, _)| (t.clone(), *price)))
                    .collect(),
            )
        } else {
            None
        }
    }

    async fn fetch_from_binance(
        &self,
        tokens: &[String],
    ) -> Result<HashMap<String, f64>, Box<dyn std::error::Error + Send + Sync>> {
        let symbols: Vec<String> = tokens.iter().map(|t| format!("{}USDT", t)).collect();

        if symbols.len() == 1 {
            let ticker: BinanceTicker = self
                .client
                .get(BINANCE_PRICE_URL)
                .query(&[("symbol", &symbols[0])])
                .send()
                .await?
                .json()
                .await?;
            let price = ticker.price.parse::<f64>()?;
            Ok(HashMap::from([(tokens[0].clone(), price)]))
        } else {
            let symbols_json = serde_json::to_string(&symbols)?;
            let tickers: Vec<BinanceTicker> = self
                .client
                .get(BINANCE_PRICE_URL)
                .query(&[("symbols", &symbols_json)])
                .send()
                .await?
                .json()
                .await?;
            Ok(tickers
                .into_iter()
                .filter_map(|t| {
                    let token = t.symbol.strip_suffix("USDT")?.to_string();
                    let price = t.price.parse::<f64>().ok()?;
                    Some((token, price))
                })
                .collect())
        }
    }
}
