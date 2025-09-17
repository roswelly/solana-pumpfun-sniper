pub mod config;
pub mod constants;
pub mod error;
pub mod price_cache;
pub mod sniper;
pub mod risk_management;
pub mod copy_trading;
pub mod jito_integration;
pub mod grpc_manager;
pub mod scam_detection;
pub mod bonding_curve;
pub mod same_block_execution;
pub mod migration_detector;

pub use config::Config;
pub use error::{Result, SniperError};
pub use price_cache::PriceCache;
pub use sniper::SniperBot;
pub use risk_management::{RiskManager, RiskConfig, RiskMetrics};
pub use copy_trading::{CopyTradingEngine, CopyTradeConfig, TraderProfile};
pub use jito_integration::{JitoManager, JitoConfig, UrgencyLevel};
pub use grpc_manager::{GrpcManager, GrpcEndpoint};
pub use scam_detection::{ScamDetector, TokenMetadata, ScamAnalysis};
pub use bonding_curve::{BondingCurveCalculator, BondingCurveState};
pub use same_block_execution::{SameBlockExecutor, SameBlockSniper, SnipeConfig};
pub use migration_detector::{MigrationDetector, Season2Features, MigrationEvent, PumpSwapMonitor};

// Generated protobuf code
pub mod geyser {
    tonic::include_proto!("geyser");
}

pub use geyser::*;
