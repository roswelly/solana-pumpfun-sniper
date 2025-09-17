use crate::error::{Result, SniperError};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraderProfile {
    pub wallet_address: Pubkey,
    pub success_rate: f64,
    pub total_trades: u32,
    pub profitable_trades: u32,
    pub average_profit: f64,
    pub last_activity: Instant,
    pub reputation_score: f64,
}

#[derive(Debug, Clone)]
pub struct CopyTradeConfig {
    pub min_success_rate: f64,
    pub min_reputation_score: f64,
    pub max_traders_to_follow: usize,
    pub copy_percentage: f64, // Percentage of trader's position to copy
    pub max_copy_amount_sol: f64,
    pub cooldown_between_copies: Duration,
}

impl Default for CopyTradeConfig {
    fn default() -> Self {
        Self {
            min_success_rate: 0.7, // 70% success rate
            min_reputation_score: 0.8, // 80% reputation
            max_traders_to_follow: 10,
            copy_percentage: 0.1, // Copy 10% of trader's position
            max_copy_amount_sol: 0.01, // Max 0.01 SOL per copy
            cooldown_between_copies: Duration::from_secs(5),
        }
    }
}

pub struct CopyTradingEngine {
    config: CopyTradeConfig,
    followed_traders: HashMap<Pubkey, TraderProfile>,
    recent_copies: HashMap<Pubkey, Instant>,
    trade_history: Vec<TradeRecord>,
}

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub trader: Pubkey,
    pub token: Pubkey,
    pub action: TradeAction,
    pub amount_sol: f64,
    pub timestamp: Instant,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub enum TradeAction {
    Buy,
    Sell,
}

impl CopyTradingEngine {
    pub fn new(config: CopyTradeConfig) -> Self {
        Self {
            config,
            followed_traders: HashMap::new(),
            recent_copies: HashMap::new(),
            trade_history: Vec::new(),
        }
    }

    pub fn add_trader(&mut self, trader: Pubkey, profile: TraderProfile) -> Result<()> {
        if self.followed_traders.len() >= self.config.max_traders_to_follow {
            return Err(SniperError::Generic(anyhow::anyhow!(
                "Maximum number of traders to follow reached"
            )));
        }

        if profile.success_rate < self.config.min_success_rate {
            return Err(SniperError::Generic(anyhow::anyhow!(
                "Trader success rate too low: {:.2}%", profile.success_rate * 100.0
            )));
        }

        if profile.reputation_score < self.config.min_reputation_score {
            return Err(SniperError::Generic(anyhow::anyhow!(
                "Trader reputation score too low: {:.2}", profile.reputation_score
            )));
        }

        self.followed_traders.insert(trader, profile);
        info!("Added trader {} to follow list", trader);
        Ok(())
    }

    pub fn remove_trader(&mut self, trader: &Pubkey) {
        if self.followed_traders.remove(trader).is_some() {
            info!("Removed trader {} from follow list", trader);
        }
    }

    pub fn should_copy_trade(&mut self, trader: &Pubkey, token: &Pubkey, action: &TradeAction, amount_sol: f64) -> Result<bool> {
        // Check if trader is being followed
        let profile = match self.followed_traders.get(trader) {
            Some(profile) => profile,
            None => return Ok(false),
        };

        // Check cooldown
        if let Some(last_copy) = self.recent_copies.get(token) {
            if last_copy.elapsed() < self.config.cooldown_between_copies {
                return Ok(false);
            }
        }

        // Check if trader meets criteria
        if profile.success_rate < self.config.min_success_rate {
            return Ok(false);
        }

        if profile.reputation_score < self.config.min_reputation_score {
            return Ok(false);
        }

        // Calculate copy amount
        let copy_amount = (amount_sol * self.config.copy_percentage).min(self.config.max_copy_amount_sol);
        
        if copy_amount <= 0.0 {
            return Ok(false);
        }

        // Record the copy trade
        self.recent_copies.insert(*token, Instant::now());
        self.trade_history.push(TradeRecord {
            trader: *trader,
            token: *token,
            action: action.clone(),
            amount_sol: copy_amount,
            timestamp: Instant::now(),
            success: false, // Will be updated later
        });

        info!("Copying trade from {}: {:?} {} SOL worth of {}", 
              trader, action, copy_amount, token);
        
        Ok(true)
    }

    pub fn update_trade_result(&mut self, trader: &Pubkey, token: &Pubkey, success: bool) {
        // Update trader profile based on trade result
        if let Some(profile) = self.followed_traders.get_mut(trader) {
            profile.total_trades += 1;
            if success {
                profile.profitable_trades += 1;
            }
            
            // Recalculate success rate
            profile.success_rate = profile.profitable_trades as f64 / profile.total_trades as f64;
            
            // Update reputation score
            profile.reputation_score = self.calculate_reputation_score(profile);
            profile.last_activity = Instant::now();
        }

        // Update trade history
        if let Some(record) = self.trade_history.iter_mut().find(|r| 
            r.trader == *trader && r.token == *token && !r.success) {
            record.success = success;
        }
    }

    fn calculate_reputation_score(&self, profile: &TraderProfile) -> f64 {
        let success_weight = 0.6;
        let activity_weight = 0.2;
        let volume_weight = 0.2;

        let success_score = profile.success_rate;
        let activity_score = if profile.last_activity.elapsed() < Duration::from_hours(24) {
            1.0
        } else if profile.last_activity.elapsed() < Duration::from_hours(72) {
            0.7
        } else {
            0.3
        };
        let volume_score = (profile.total_trades as f64 / 100.0).min(1.0);

        success_score * success_weight + activity_score * activity_weight + volume_score * volume_weight
    }

    pub fn get_top_traders(&self, limit: usize) -> Vec<(Pubkey, TraderProfile)> {
        let mut traders: Vec<_> = self.followed_traders.iter().collect();
        traders.sort_by(|a, b| b.1.reputation_score.partial_cmp(&a.1.reputation_score).unwrap());
        
        traders.into_iter()
            .take(limit)
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    pub fn analyze_trader_performance(&self, trader: &Pubkey) -> Option<TraderAnalysis> {
        let profile = self.followed_traders.get(trader)?;
        
        let recent_trades: Vec<_> = self.trade_history
            .iter()
            .filter(|r| r.trader == *trader && r.timestamp.elapsed() < Duration::from_hours(24))
            .collect();

        let recent_success_rate = if recent_trades.is_empty() {
            0.0
        } else {
            recent_trades.iter().filter(|r| r.success).count() as f64 / recent_trades.len() as f64
        };

        Some(TraderAnalysis {
            trader: *trader,
            overall_success_rate: profile.success_rate,
            recent_success_rate,
            total_trades: profile.total_trades,
            recent_trades: recent_trades.len(),
            reputation_score: profile.reputation_score,
            recommendation: if recent_success_rate > 0.8 {
                "Strong buy"
            } else if recent_success_rate > 0.6 {
                "Moderate"
            } else {
                "Caution"
            }.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TraderAnalysis {
    pub trader: Pubkey,
    pub overall_success_rate: f64,
    pub recent_success_rate: f64,
    pub total_trades: u32,
    pub recent_trades: usize,
    pub reputation_score: f64,
    pub recommendation: String,
}

pub struct TraderDiscovery {
    known_good_traders: Vec<Pubkey>,
}

impl TraderDiscovery {
    pub fn new() -> Self {
        Self {
            known_good_traders: vec![
                // Add known successful traders here
                // These would be discovered through analysis of successful trades
            ],
        }
    }

    pub fn discover_traders_from_transactions(&mut self, transactions: &[TransactionData]) -> Vec<Pubkey> {
        let mut trader_stats: HashMap<Pubkey, TraderStats> = HashMap::new();

        for tx in transactions {
            if let Some(trader) = &tx.trader {
                let stats = trader_stats.entry(*trader).or_insert(TraderStats::new());
                stats.add_trade(tx.success, tx.profit);
            }
        }

        // Find traders with good performance
        trader_stats.into_iter()
            .filter(|(_, stats)| stats.success_rate > 0.7 && stats.total_trades > 10)
            .map(|(trader, _)| trader)
            .collect()
    }
}

#[derive(Debug)]
struct TraderStats {
    total_trades: u32,
    successful_trades: u32,
    total_profit: f64,
}

impl TraderStats {
    fn new() -> Self {
        Self {
            total_trades: 0,
            successful_trades: 0,
            total_profit: 0.0,
        }
    }

    fn add_trade(&mut self, success: bool, profit: f64) {
        self.total_trades += 1;
        if success {
            self.successful_trades += 1;
        }
        self.total_profit += profit;
    }

    fn success_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            self.successful_trades as f64 / self.total_trades as f64
        }
    }
}

#[derive(Debug)]
pub struct TransactionData {
    pub trader: Option<Pubkey>,
    pub token: Pubkey,
    pub success: bool,
    pub profit: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_trading_engine() {
        let config = CopyTradeConfig::default();
        let mut engine = CopyTradingEngine::new(config);
        
        let trader = Pubkey::new_unique();
        let profile = TraderProfile {
            wallet_address: trader,
            success_rate: 0.8,
            total_trades: 100,
            profitable_trades: 80,
            average_profit: 0.05,
            last_activity: Instant::now(),
            reputation_score: 0.9,
        };

        assert!(engine.add_trader(trader, profile).is_ok());
        
        let token = Pubkey::new_unique();
        let should_copy = engine.should_copy_trade(&trader, &token, &TradeAction::Buy, 0.1);
        assert!(should_copy.is_ok() && should_copy.unwrap());
    }
}
