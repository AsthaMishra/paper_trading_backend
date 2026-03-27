use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Deserialize)]
struct JupiterResponse {
    data: HashMap<String, JupiterTokenData>,
}

#[derive(Deserialize)]
struct JupiterTokenData {
    price: f64,
}

#[derive(Clone)]
pub struct MarketDataService {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, (f64, Instant)>>>,
    cache_ttl: Duration,
}

impl MarketDataService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(5),
        }
    }   

    pub async fn get_prices(
        &self,
        tokens: &[String],
    ) -> Result<HashMap<String, f64>, Box<dyn std::error::Error + Send + Sync>> {
        if tokens.is_empty() {
            return Ok(HashMap::new());
        }

        // Return from cache if all tokens are fresh
        {
            let cache = self.cache.read().await;
            let now = Instant::now();
            let all_cached = tokens.iter().all(|t| {
                cache
                    .get(t)
                    .map(|(_, ts)| now.duration_since(*ts) < self.cache_ttl)
                    .unwrap_or(false)
            });
            if all_cached {
                return Ok(tokens
                    .iter()
                    .filter_map(|t| cache.get(t).map(|(price, _)| (t.clone(), *price)))
                    .collect());
            }
        }

        let ids = tokens.join(",");
        let url = format!("https://price.jup.ag/v6/price?ids={}", ids);
        let response: JupiterResponse = self.client.get(&url).send().await?.json().await?;

        let mut result = HashMap::new();
        let mut cache = self.cache.write().await;
        let now = Instant::now();

        for (symbol, data) in response.data {
            result.insert(symbol.clone(), data.price);
            cache.insert(symbol, (data.price, now));
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
}
