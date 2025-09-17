use crate::constants::*;
use crate::error::{Result, SniperError};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{warn, info};

#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub market_cap: f64,
    pub liquidity: f64,
    pub volume_24h: f64,
    pub holder_count: u32,
    pub is_honeypot: bool,
    pub rug_pull_score: f64, // 0.0 = safe, 1.0 = high risk
    pub creation_time: Instant,
}

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_rug_pull_score: f64,
    pub min_liquidity_sol: f64,
    pub min_holder_count: u32,
    pub max_slippage_percentage: f64,
    pub max_buy_amount_sol: f64,
    pub cooldown_period: Duration,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_rug_pull_score: 0.3, // Allow up to 30% risk
            min_liquidity_sol: MIN_LIQUIDITY_THRESHOLD,
            min_holder_count: 10,
            max_slippage_percentage: MAX_SLIPPAGE_PERCENTAGE,
            max_buy_amount_sol: MAX_BUY_AMOUNT_SOL,
            cooldown_period: Duration::from_secs(30),
        }
    }
}

pub struct RiskManager {
    config: RiskConfig,
    recent_trades: HashMap<Pubkey, Instant>,
    blacklisted_tokens: std::collections::HashSet<Pubkey>,
    honeypot_detector: HoneypotDetector,
}

impl RiskManager {
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            recent_trades: HashMap::new(),
            blacklisted_tokens: std::collections::HashSet::new(),
            honeypot_detector: HoneypotDetector::new(),
        }
    }

    pub fn evaluate_token(&mut self, mint: &Pubkey, metrics: &RiskMetrics) -> Result<bool> {
        // Check if token is blacklisted
        if self.blacklisted_tokens.contains(mint) {
            warn!("Token {} is blacklisted", mint);
            return Ok(false);
        }

        // Check cooldown period
        if let Some(last_trade) = self.recent_trades.get(mint) {
            if last_trade.elapsed() < self.config.cooldown_period {
                warn!("Token {} is in cooldown period", mint);
                return Ok(false);
            }
        }

        // Check rug pull score
        if metrics.rug_pull_score > self.config.max_rug_pull_score {
            warn!("Token {} has high rug pull score: {:.2}", mint, metrics.rug_pull_score);
            self.blacklisted_tokens.insert(*mint);
            return Ok(false);
        }

        // Check liquidity
        if metrics.liquidity < self.config.min_liquidity_sol {
            warn!("Token {} has insufficient liquidity: {:.2} SOL", mint, metrics.liquidity);
            return Ok(false);
        }

        // Check holder count
        if metrics.holder_count < self.config.min_holder_count {
            warn!("Token {} has too few holders: {}", mint, metrics.holder_count);
            return Ok(false);
        }

        // Check for honeypot
        if metrics.is_honeypot {
            warn!("Token {} detected as honeypot", mint);
            self.blacklisted_tokens.insert(*mint);
            return Ok(false);
        }

        // Additional AI-powered checks
        if self.honeypot_detector.is_suspicious(metrics) {
            warn!("Token {} flagged as suspicious by AI detector", mint);
            return Ok(false);
        }

        info!("Token {} passed all risk checks", mint);
        Ok(true)
    }

    pub fn record_trade(&mut self, mint: &Pubkey) {
        self.recent_trades.insert(*mint, Instant::now());
    }

    pub fn calculate_optimal_buy_amount(&self, metrics: &RiskMetrics, available_sol: f64) -> f64 {
        let base_amount = self.config.max_buy_amount_sol.min(available_sol);
        
        // Adjust based on risk metrics
        let risk_multiplier = 1.0 - metrics.rug_pull_score;
        let liquidity_multiplier = (metrics.liquidity / self.config.min_liquidity_sol).min(1.0);
        
        base_amount * risk_multiplier * liquidity_multiplier
    }

    pub fn should_stop_loss(&self, entry_price: f64, current_price: f64, stop_loss_percentage: f64) -> bool {
        let price_change = (current_price - entry_price) / entry_price;
        price_change <= -stop_loss_percentage
    }

    pub fn should_take_profit(&self, entry_price: f64, current_price: f64, take_profit_percentage: f64) -> bool {
        let price_change = (current_price - entry_price) / entry_price;
        price_change >= take_profit_percentage
    }
}

pub struct HoneypotDetector {
    suspicious_patterns: Vec<String>,
}

impl HoneypotDetector {
    pub fn new() -> Self {
        Self {
            suspicious_patterns: vec![
                "test".to_string(),
                "fake".to_string(),
                "scam".to_string(),
                "rug".to_string(),
                "honeypot".to_string(),
            ],
        }
    }

    pub fn is_suspicious(&self, metrics: &RiskMetrics) -> bool {
        // Check for suspicious patterns in token metadata
        // This is a simplified version - in production, you'd analyze token metadata
        
        // Check for extremely high initial liquidity (potential honeypot)
        if metrics.liquidity > 1000.0 && metrics.holder_count < 5 {
            return true;
        }

        // Check for suspicious volume patterns
        if metrics.volume_24h > 10000.0 && metrics.holder_count < 10 {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_evaluation() {
        let config = RiskConfig::default();
        let mut risk_manager = RiskManager::new(config);
        
        let mint = Pubkey::new_unique();
        let metrics = RiskMetrics {
            market_cap: 10000.0,
            liquidity: 2000.0,
            volume_24h: 5000.0,
            holder_count: 20,
            is_honeypot: false,
            rug_pull_score: 0.1,
            creation_time: Instant::now(),
        };

        assert!(risk_manager.evaluate_token(&mint, &metrics).unwrap());
    }

    #[test]
    fn test_optimal_buy_amount() {
        let config = RiskConfig::default();
        let risk_manager = RiskManager::new(config);
        
        let metrics = RiskMetrics {
            market_cap: 10000.0,
            liquidity: 2000.0,
            volume_24h: 5000.0,
            holder_count: 20,
            is_honeypot: false,
            rug_pull_score: 0.1,
            creation_time: Instant::now(),
        };

        let amount = risk_manager.calculate_optimal_buy_amount(&metrics, 1.0);
        assert!(amount > 0.0 && amount <= 1.0);
    }
}
