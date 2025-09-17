use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub buyer_private_key: String,
    pub grpc_endpoint: String,
    pub grpc_auth_token: String,
    pub solana_rpc_endpoint: String,
    pub market_cap_threshold_usd: f64,
    pub buy_amount_sol: f64,
    
    // New features configuration
    pub enable_jito: bool,
    pub enable_copy_trading: bool,
    pub enable_scam_detection: bool,
    pub enable_same_block_execution: bool,
    pub enable_risk_management: bool,
    pub max_slippage_percentage: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
    pub copy_trading_percentage: f64,
    pub jito_tip_lamports: u64,
    
    // Season 2 Features
    pub enable_migration_detection: bool,
    pub enable_pump_swap_monitoring: bool,
    pub enable_creator_revenue_tracking: bool,
    pub migration_threshold: f64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok(); // Load .env file if it exists

        let buyer_private_key = env::var("BUYER_PRIVATE_KEY_PATH")
            .map_err(|_| anyhow!("BUYER_PRIVATE_KEY_PATH environment variable not set"))?;

        let grpc_endpoint = env::var("GRPC_ENDPOINT")
            .map_err(|_| anyhow!("GRPC_ENDPOINT environment variable not set"))?;

        let grpc_auth_token = env::var("GRPC_AUTH_TOKEN")
            .map_err(|_| anyhow!("GRPC_AUTH_TOKEN environment variable not set"))?;

        let solana_rpc_endpoint = if let Ok(endpoint) = env::var("SOLANA_RPC_ENDPOINT") {
            endpoint
        } else if let Ok(api_key) = env::var("HELIUS_API_KEY") {
            format!("https://pomaded-lithotomies-xfbhnqagbt-dedicated.helius-rpc.com/?api-key={}", api_key)
        } else {
            return Err(anyhow!("Missing HELIUS_API_KEY or SOLANA_RPC_ENDPOINT"));
        };

        let market_cap_threshold_usd = env::var("MARKET_CAP_THRESHOLD_USD")
            .unwrap_or_else(|_| "8000.0".to_string())
            .parse()
            .map_err(|_| anyhow!("Invalid MARKET_CAP_THRESHOLD_USD value"))?;

        let buy_amount_sol = env::var("BUY_AMOUNT_SOL")
            .unwrap_or_else(|_| "0.001".to_string())
            .parse()
            .map_err(|_| anyhow!("Invalid BUY_AMOUNT_SOL value"))?;

        // New features configuration
        let enable_jito = env::var("ENABLE_JITO")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let enable_copy_trading = env::var("ENABLE_COPY_TRADING")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let enable_scam_detection = env::var("ENABLE_SCAM_DETECTION")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let enable_same_block_execution = env::var("ENABLE_SAME_BLOCK_EXECUTION")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let enable_risk_management = env::var("ENABLE_RISK_MANAGEMENT")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let max_slippage_percentage = env::var("MAX_SLIPPAGE_PERCENTAGE")
            .unwrap_or_else(|_| "20.0".to_string())
            .parse()
            .unwrap_or(20.0);

        let stop_loss_percentage = env::var("STOP_LOSS_PERCENTAGE")
            .unwrap_or_else(|_| "10.0".to_string())
            .parse()
            .unwrap_or(10.0);

        let take_profit_percentage = env::var("TAKE_PROFIT_PERCENTAGE")
            .unwrap_or_else(|_| "50.0".to_string())
            .parse()
            .unwrap_or(50.0);

        let copy_trading_percentage = env::var("COPY_TRADING_PERCENTAGE")
            .unwrap_or_else(|_| "10.0".to_string())
            .parse()
            .unwrap_or(10.0);

        let jito_tip_lamports = env::var("JITO_TIP_LAMPORTS")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .unwrap_or(10000);

        // Season 2 Features
        let enable_migration_detection = env::var("ENABLE_MIGRATION_DETECTION")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let enable_pump_swap_monitoring = env::var("ENABLE_PUMP_SWAP_MONITORING")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let enable_creator_revenue_tracking = env::var("ENABLE_CREATOR_REVENUE_TRACKING")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let migration_threshold = env::var("MIGRATION_THRESHOLD")
            .unwrap_or_else(|_| "0.95".to_string())
            .parse()
            .unwrap_or(0.95);

        Ok(Config {
            buyer_private_key,
            grpc_endpoint,
            grpc_auth_token,
            solana_rpc_endpoint,
            market_cap_threshold_usd,
            buy_amount_sol,
            enable_jito,
            enable_copy_trading,
            enable_scam_detection,
            enable_same_block_execution,
            enable_risk_management,
            max_slippage_percentage,
            stop_loss_percentage,
            take_profit_percentage,
            copy_trading_percentage,
            jito_tip_lamports,
            enable_migration_detection,
            enable_pump_swap_monitoring,
            enable_creator_revenue_tracking,
            migration_threshold,
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Validate private key format (basic check)
        if self.buyer_private_key.len() < 32 {
            return Err(anyhow!("Invalid private key format"));
        }

        // Validate URLs
        if !self.grpc_endpoint.starts_with("http") {
            return Err(anyhow!("Invalid gRPC endpoint URL"));
        }

        if !self.solana_rpc_endpoint.starts_with("http") {
            return Err(anyhow!("Invalid Solana RPC endpoint URL"));
        }

        // Validate numeric values
        if self.market_cap_threshold_usd <= 0.0 {
            return Err(anyhow!("Market cap threshold must be positive"));
        }

        if self.buy_amount_sol <= 0.0 {
            return Err(anyhow!("Buy amount must be positive"));
        }

        Ok(())
    }
}
