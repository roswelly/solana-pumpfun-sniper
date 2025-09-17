use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

#[derive(Deserialize)]
struct CoinGeckoResponse {
    solana: SolanaPrice,
}

#[derive(Deserialize)]
struct SolanaPrice {
    usd: f64,
}

pub struct PriceCache {
    price: Arc<RwLock<f64>>,
}

impl PriceCache {
    pub fn new() -> Self {
        Self {
            price: Arc::new(RwLock::new(0.0)),
        }
    }

    pub fn get(&self) -> f64 {
        *self.price.read()
    }

    pub fn set(&self, price: f64) {
        *self.price.write() = price;
    }

    async fn fetch_sol_price() -> Result<f64> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("CoinGecko API returned error status: {}", response.status()));
        }

        let data: CoinGeckoResponse = response.json().await?;
        
        if data.solana.usd == 0.0 {
            return Err(anyhow!("CoinGecko returned zero price for SOL"));
        }

        Ok(data.solana.usd)
    }

    pub async fn update_price_periodically(&self) {
        let price_cache = Arc::new(self.price.clone());
        
        // Initial fetch
        match Self::fetch_sol_price().await {
            Ok(price) => {
                *price_cache.write() = price;
                info!("SOL Price updated: ${:.2}", price);
            }
            Err(e) => {
                error!("CoinGecko price fetch failed: {}. Price not updated.", e);
            }
        }

        // Periodic updates every 30 seconds
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            
            match Self::fetch_sol_price().await {
                Ok(price) => {
                    *price_cache.write() = price;
                    info!("SOL Price updated: ${:.2}", price);
                }
                Err(e) => {
                    error!("CoinGecko price fetch failed: {}. Price not updated.", e);
                }
            }
        }
    }
}

impl Default for PriceCache {
    fn default() -> Self {
        Self::new()
    }
}
