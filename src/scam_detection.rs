use crate::error::{Result, SniperError};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_uri: String,
    pub creator: Pubkey,
    pub creation_time: Instant,
    pub initial_supply: u64,
    pub decimals: u8,
}

#[derive(Debug, Clone)]
pub struct ScamAnalysis {
    pub mint: Pubkey,
    pub scam_score: f64, // 0.0 = safe, 1.0 = definitely scam
    pub risk_factors: Vec<RiskFactor>,
    pub recommendation: ScamRecommendation,
    pub confidence: f64,
    pub analysis_time: Instant,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub severity: f64, // 0.0 to 1.0
    pub description: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RiskFactorType {
    SuspiciousName,
    DuplicateMetadata,
    HoneypotPattern,
    RugPullIndicator,
    LowLiquidity,
    SuspiciousCreator,
    UnusualTradingPattern,
    MetadataAnomaly,
    SocialMediaRedFlags,
    CodeAnalysis,
}

#[derive(Debug, Clone)]
pub enum ScamRecommendation {
    Safe,
    Caution,
    HighRisk,
    Avoid,
}

pub struct ScamDetector {
    known_scam_patterns: HashMap<String, f64>,
    suspicious_creators: std::collections::HashSet<Pubkey>,
    analyzed_tokens: HashMap<Pubkey, ScamAnalysis>,
    ml_model: MLModel,
}

impl ScamDetector {
    pub fn new() -> Self {
        let mut known_patterns = HashMap::new();
        
        // Add known scam patterns
        known_patterns.insert("test".to_string(), 0.8);
        known_patterns.insert("fake".to_string(), 0.9);
        known_patterns.insert("scam".to_string(), 0.95);
        known_patterns.insert("rug".to_string(), 0.9);
        known_patterns.insert("honeypot".to_string(), 0.95);
        known_patterns.insert("pump".to_string(), 0.3);
        known_patterns.insert("moon".to_string(), 0.2);
        known_patterns.insert("doge".to_string(), 0.1);

        Self {
            known_scam_patterns: known_patterns,
            suspicious_creators: std::collections::HashSet::new(),
            analyzed_tokens: HashMap::new(),
            ml_model: MLModel::new(),
        }
    }

    pub async fn analyze_token(&mut self, metadata: &TokenMetadata, trading_data: &TradingData) -> ScamAnalysis {
        let mut risk_factors = Vec::new();
        let mut total_score = 0.0;
        let mut confidence = 0.0;

        // Check name patterns
        if let Some(score) = self.check_name_patterns(&metadata.name, &metadata.symbol) {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::SuspiciousName,
                severity: score,
                description: "Suspicious name or symbol detected".to_string(),
                evidence: vec![format!("Name: {}", metadata.name), format!("Symbol: {}", metadata.symbol)],
            });
            total_score += score * 0.2;
            confidence += 0.2;
        }

        // Check creator reputation
        if self.suspicious_creators.contains(&metadata.creator) {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::SuspiciousCreator,
                severity: 0.9,
                description: "Creator is known for suspicious activity".to_string(),
                evidence: vec![format!("Creator: {}", metadata.creator)],
            });
            total_score += 0.9 * 0.3;
            confidence += 0.3;
        }

        // Check liquidity patterns
        if let Some(score) = self.check_liquidity_patterns(trading_data) {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::LowLiquidity,
                severity: score,
                description: "Suspicious liquidity patterns detected".to_string(),
                evidence: vec![format!("Liquidity: {} SOL", trading_data.liquidity)],
            });
            total_score += score * 0.15;
            confidence += 0.15;
        }

        // Check trading patterns
        if let Some(score) = self.check_trading_patterns(trading_data) {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::UnusualTradingPattern,
                severity: score,
                description: "Unusual trading patterns detected".to_string(),
                evidence: vec![format!("Volume: {}", trading_data.volume_24h)],
            });
            total_score += score * 0.2;
            confidence += 0.2;
        }

        // Check metadata anomalies
        if let Some(score) = self.check_metadata_anomalies(metadata) {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::MetadataAnomaly,
                severity: score,
                description: "Metadata anomalies detected".to_string(),
                evidence: vec![format!("Description length: {}", metadata.description.len())],
            });
            total_score += score * 0.1;
            confidence += 0.1;
        }

        // ML-based analysis
        let ml_score = self.ml_model.predict_scam_probability(metadata, trading_data);
        if ml_score > 0.5 {
            risk_factors.push(RiskFactor {
                factor_type: RiskFactorType::CodeAnalysis,
                severity: ml_score,
                description: "ML model detected suspicious patterns".to_string(),
                evidence: vec![format!("ML Score: {:.2}", ml_score)],
            });
            total_score += ml_score * 0.25;
            confidence += 0.25;
        }

        // Normalize score
        let scam_score = total_score.min(1.0);
        confidence = confidence.min(1.0);

        let recommendation = match scam_score {
            s if s < 0.2 => ScamRecommendation::Safe,
            s if s < 0.5 => ScamRecommendation::Caution,
            s if s < 0.8 => ScamRecommendation::HighRisk,
            _ => ScamRecommendation::Avoid,
        };

        let analysis = ScamAnalysis {
            mint: metadata.mint,
            scam_score,
            risk_factors,
            recommendation,
            confidence,
            analysis_time: Instant::now(),
        };

        self.analyzed_tokens.insert(metadata.mint, analysis.clone());
        analysis
    }

    fn check_name_patterns(&self, name: &str, symbol: &str) -> Option<f64> {
        let text = format!("{} {}", name.to_lowercase(), symbol.to_lowercase());
        
        for (pattern, score) in &self.known_scam_patterns {
            if text.contains(pattern) {
                return Some(*score);
            }
        }
        
        None
    }

    fn check_liquidity_patterns(&self, trading_data: &TradingData) -> Option<f64> {
        // Check for suspicious liquidity patterns
        if trading_data.liquidity < 100.0 && trading_data.volume_24h > 10000.0 {
            return Some(0.8); // High volume with low liquidity is suspicious
        }
        
        if trading_data.liquidity > 10000.0 && trading_data.holder_count < 10 {
            return Some(0.7); // High liquidity with few holders
        }
        
        None
    }

    fn check_trading_patterns(&self, trading_data: &TradingData) -> Option<f64> {
        // Check for pump and dump patterns
        if trading_data.price_change_24h > 1000.0 && trading_data.holder_count < 50 {
            return Some(0.9); // Extreme price increase with few holders
        }
        
        if trading_data.volume_24h > 100000.0 && trading_data.holder_count < 20 {
            return Some(0.8); // High volume with few holders
        }
        
        None
    }

    fn check_metadata_anomalies(&self, metadata: &TokenMetadata) -> Option<f64> {
        // Check for suspicious metadata
        if metadata.description.len() < 10 {
            return Some(0.6); // Very short description
        }
        
        if metadata.name.len() > 50 {
            return Some(0.4); // Very long name
        }
        
        if metadata.symbol.len() > 10 {
            return Some(0.5); // Very long symbol
        }
        
        None
    }

    pub fn add_suspicious_creator(&mut self, creator: Pubkey) {
        self.suspicious_creators.insert(creator);
        info!("Added suspicious creator: {}", creator);
    }

    pub fn get_analysis(&self, mint: &Pubkey) -> Option<&ScamAnalysis> {
        self.analyzed_tokens.get(mint)
    }

    pub fn is_token_safe(&self, mint: &Pubkey) -> bool {
        if let Some(analysis) = self.analyzed_tokens.get(mint) {
            matches!(analysis.recommendation, ScamRecommendation::Safe | ScamRecommendation::Caution)
        } else {
            true // Assume safe if not analyzed
        }
    }

    pub fn get_scam_score(&self, mint: &Pubkey) -> Option<f64> {
        self.analyzed_tokens.get(mint).map(|a| a.scam_score)
    }
}

#[derive(Debug, Clone)]
pub struct TradingData {
    pub mint: Pubkey,
    pub liquidity: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub holder_count: u32,
    pub transaction_count: u32,
    pub market_cap: f64,
    pub last_update: Instant,
}

pub struct MLModel {
    // In a real implementation, this would contain an actual ML model
    // For now, it's a placeholder with rule-based logic
}

impl MLModel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn predict_scam_probability(&self, metadata: &TokenMetadata, trading_data: &TradingData) -> f64 {
        let mut score = 0.0;
        
        // Rule-based ML simulation
        if metadata.description.contains("guaranteed") {
            score += 0.3;
        }
        
        if trading_data.price_change_24h > 500.0 {
            score += 0.4;
        }
        
        if trading_data.holder_count < 5 {
            score += 0.5;
        }
        
        if metadata.name.len() < 3 {
            score += 0.2;
        }
        
        score.min(1.0)
    }
}

pub struct HoneypotDetector {
    known_honeypot_patterns: Vec<String>,
}

impl HoneypotDetector {
    pub fn new() -> Self {
        Self {
            known_honeypot_patterns: vec![
                "test".to_string(),
                "fake".to_string(),
                "scam".to_string(),
            ],
        }
    }

    pub fn detect_honeypot(&self, metadata: &TokenMetadata, trading_data: &TradingData) -> bool {
        // Check for honeypot patterns
        let text = format!("{} {}", metadata.name.to_lowercase(), metadata.symbol.to_lowercase());
        
        for pattern in &self.known_honeypot_patterns {
            if text.contains(pattern) {
                return true;
            }
        }
        
        // Check for honeypot trading patterns
        if trading_data.liquidity > 1000.0 && trading_data.holder_count < 3 {
            return true;
        }
        
        false
    }
}

pub struct RugPullDetector {
    suspicious_patterns: HashMap<String, f64>,
}

impl RugPullDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        patterns.insert("rug".to_string(), 0.9);
        patterns.insert("pull".to_string(), 0.9);
        patterns.insert("exit".to_string(), 0.7);
        patterns.insert("scam".to_string(), 0.95);
        
        Self {
            suspicious_patterns: patterns,
        }
    }

    pub fn detect_rug_pull(&self, metadata: &TokenMetadata, trading_data: &TradingData) -> f64 {
        let mut score = 0.0;
        
        // Check name patterns
        let text = format!("{} {}", metadata.name.to_lowercase(), metadata.symbol.to_lowercase());
        for (pattern, pattern_score) in &self.suspicious_patterns {
            if text.contains(pattern) {
                score += pattern_score;
            }
        }
        
        // Check trading patterns
        if trading_data.price_change_24h < -90.0 {
            score += 0.8; // Extreme price drop
        }
        
        if trading_data.volume_24h > 50000.0 && trading_data.holder_count < 10 {
            score += 0.7; // High volume with few holders
        }
        
        score.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scam_detector() {
        let mut detector = ScamDetector::new();
        
        let metadata = TokenMetadata {
            mint: Pubkey::new_unique(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "This is a test token".to_string(),
            image_uri: "https://example.com/image.png".to_string(),
            creator: Pubkey::new_unique(),
            creation_time: Instant::now(),
            initial_supply: 1000000,
            decimals: 6,
        };
        
        let trading_data = TradingData {
            mint: metadata.mint,
            liquidity: 1000.0,
            volume_24h: 5000.0,
            price_change_24h: 10.0,
            holder_count: 50,
            transaction_count: 100,
            market_cap: 10000.0,
            last_update: Instant::now(),
        };
        
        let analysis = futures::executor::block_on(detector.analyze_token(&metadata, &trading_data));
        assert!(analysis.scam_score >= 0.0 && analysis.scam_score <= 1.0);
    }

    #[test]
    fn test_honeypot_detector() {
        let detector = HoneypotDetector::new();
        
        let metadata = TokenMetadata {
            mint: Pubkey::new_unique(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "This is a test token".to_string(),
            image_uri: "https://example.com/image.png".to_string(),
            creator: Pubkey::new_unique(),
            creation_time: Instant::now(),
            initial_supply: 1000000,
            decimals: 6,
        };
        
        let trading_data = TradingData {
            mint: metadata.mint,
            liquidity: 2000.0,
            volume_24h: 1000.0,
            price_change_24h: 5.0,
            holder_count: 2,
            transaction_count: 10,
            market_cap: 5000.0,
            last_update: Instant::now(),
        };
        
        assert!(detector.detect_honeypot(&metadata, &trading_data));
    }
}
