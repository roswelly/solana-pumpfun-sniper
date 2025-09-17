# Solana PumpFun Sniper Bot - Updated Documentation (2024)

## 🚀 Project Overview

This is a cutting-edge Rust implementation of a Solana PumpFun sniper bot that has been updated with the latest 2024 features and improvements. The bot monitors new token mints on PumpFun and automatically executes trades with advanced risk management, AI-powered scam detection, and ultra-fast execution capabilities.

## 📋 Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Components](#core-components)
3. [Advanced Features (2024 Updates)](#advanced-features-2024-updates)
4. [Configuration Guide](#configuration-guide)
5. [Setup & Installation](#setup--installation)
6. [Usage Examples](#usage-examples)
7. [Performance Optimization](#performance-optimization)
8. [Security Features](#security-features)
9. [Troubleshooting](#troubleshooting)
10. [API Reference](#api-reference)

## 🏗️ Architecture Overview

### System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   gRPC Manager  │────│  Same-Block     │────│   Jito Client    │
│  (Multi-conn)   │    │   Executor      │    │  (MEV Protect)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Scam Detection │    │  Risk Manager   │    │ Copy Trading    │
│   (AI-Powered)  │    │  (Portfolio)    │    │    Engine       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Bonding Curve   │    │  Price Cache    │    │   Main Sniper   │
│  Calculator     │    │   (SOL/USD)     │    │     Bot         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Data Flow

1. **Real-time Monitoring**: Multiple gRPC connections monitor PumpFun transactions
2. **Transaction Analysis**: AI-powered analysis detects scams and evaluates risk
3. **Market Analysis**: Bonding curve calculations determine optimal entry points
4. **Risk Assessment**: Advanced risk management evaluates trade viability
5. **Execution**: Same-block execution with Jito MEV protection
6. **Monitoring**: Continuous performance tracking and optimization

## 🔧 Core Components

### 1. Main Application (`main.rs`)
- **Purpose**: Application entry point and orchestration
- **Key Features**:
  - Environment variable loading
  - Component initialization
  - Error handling and logging
  - Graceful shutdown

### 2. Configuration Management (`config.rs`)
- **Purpose**: Centralized configuration management
- **Key Features**:
  - Environment variable parsing
  - Configuration validation
  - Default value management
  - Feature toggles

### 3. Price Cache (`price_cache.rs`)
- **Purpose**: SOL price fetching and caching
- **Key Features**:
  - CoinGecko API integration
  - Periodic price updates (30s intervals)
  - Thread-safe caching with RwLock
  - Error handling and fallback

### 4. Error Handling (`error.rs`)
- **Purpose**: Comprehensive error management
- **Key Features**:
  - Custom error types
  - Error chaining
  - Context preservation
  - Structured error reporting

### 5. Constants (`constants.rs`)
- **Purpose**: Application constants and known addresses
- **Key Features**:
  - PumpFun program IDs
  - Discriminators for instruction identification
  - Bonding curve constants
  - Risk management thresholds

## 🚀 Advanced Features (2024 Updates)

### 1. Same-Block Execution (`same_block_execution.rs`)

**Purpose**: Achieve near-instantaneous trade execution within the same Solana block.

**Key Components**:
- `SameBlockExecutor`: Manages transaction scheduling and execution
- `BlockTracker`: Real-time block monitoring and hash caching
- `ExecutionQueue`: Priority-based transaction queuing
- `SameBlockSniper`: High-speed sniping with block targeting

**Features**:
- Sub-100ms execution times
- Block-aware transaction scheduling
- Priority-based execution queue
- Automatic retry mechanisms
- Real-time block hash caching

**Usage Example**:
```rust
let sniper = SameBlockSniper::new(rpc_client, snipe_config);
let signature = sniper.snipe_token(instructions, signers, fee_payer).await?;
```

### 2. Jito Integration (`jito_integration.rs`)

**Purpose**: Ultra-fast transaction processing with MEV protection.

**Key Components**:
- `JitoClient`: Handles Jito transaction submission
- `JitoManager`: Manages Jito configuration and optimization
- `JitoBundleBuilder`: Creates transaction bundles
- `UrgencyLevel`: Priority-based tip calculation

**Features**:
- MEV protection against front-running
- Dynamic tip calculation based on network congestion
- Transaction bundling for efficiency
- Automatic fallback to regular transactions
- Network congestion monitoring

**Configuration**:
```rust
let jito_config = JitoConfig {
    enabled: true,
    default_tip_lamports: 10000,
    max_tip_lamports: 100000,
    tip_strategy: TipStrategy::Dynamic(DynamicTipConfig {
        base_tip: 5000,
        network_congestion_multiplier: 1.5,
        urgency_multiplier: 2.0,
    }),
};
```

### 3. Multiple gRPC Connections (`grpc_manager.rs`)

**Purpose**: Redundant connection management for maximum reliability.

**Key Components**:
- `GrpcManager`: Manages multiple gRPC connections
- `GrpcEndpoint`: Individual endpoint configuration
- `LoadBalancer`: Intelligent load distribution
- `ConnectionStats`: Real-time connection monitoring

**Features**:
- Automatic failover between connections
- Load balancing across healthy connections
- Health monitoring and recovery
- Connection pooling and optimization
- Real-time statistics and monitoring

**Configuration**:
```rust
let endpoints = vec![
    GrpcEndpoint {
        url: "https://endpoint1.com".to_string(),
        auth_token: "token1".to_string(),
        priority: 1,
        weight: 1.0,
        enabled: true,
    },
    GrpcEndpoint {
        url: "https://endpoint2.com".to_string(),
        auth_token: "token2".to_string(),
        priority: 2,
        weight: 0.8,
        enabled: true,
    },
];
```

### 4. AI-Powered Scam Detection (`scam_detection.rs`)

**Purpose**: Advanced scam and honeypot detection using machine learning.

**Key Components**:
- `ScamDetector`: Main detection engine
- `TokenMetadata`: Comprehensive token analysis
- `ScamAnalysis`: Risk assessment results
- `HoneypotDetector`: Honeypot pattern recognition
- `RugPullDetector`: Rug pull pattern detection

**Features**:
- ML-based scam probability calculation
- Pattern recognition for known scams
- Metadata anomaly detection
- Trading pattern analysis
- Real-time risk scoring

**Risk Factors Analyzed**:
- Suspicious name patterns
- Duplicate metadata
- Honeypot indicators
- Rug pull patterns
- Low liquidity warnings
- Suspicious creator addresses
- Unusual trading patterns
- Metadata anomalies

**Usage Example**:
```rust
let mut detector = ScamDetector::new();
let analysis = detector.analyze_token(&metadata, &trading_data).await;
match analysis.recommendation {
    ScamRecommendation::Safe => { /* Proceed with trade */ }
    ScamRecommendation::Avoid => { /* Skip trade */ }
    _ => { /* Evaluate further */ }
}
```

### 5. Advanced Risk Management (`risk_management.rs`)

**Purpose**: Comprehensive portfolio risk assessment and management.

**Key Components**:
- `RiskManager`: Main risk assessment engine
- `RiskMetrics`: Token risk evaluation
- `RiskConfig`: Risk management configuration
- `HoneypotDetector`: Honeypot detection
- `RiskFactor`: Individual risk factors

**Features**:
- Real-time risk scoring
- Portfolio diversification analysis
- Stop-loss and take-profit automation
- Position sizing optimization
- Risk factor correlation analysis

**Risk Metrics**:
- Market cap analysis
- Liquidity assessment
- Volume pattern analysis
- Holder count evaluation
- Rug pull probability
- Honeypot detection

**Configuration**:
```rust
let risk_config = RiskConfig {
    max_rug_pull_score: 0.3,
    min_liquidity_sol: 1000.0,
    min_holder_count: 10,
    max_slippage_percentage: 20.0,
    max_buy_amount_sol: 0.1,
    cooldown_period: Duration::from_secs(30),
};
```

### 6. Intelligent Copy Trading (`copy_trading.rs`)

**Purpose**: Automatically mirror successful traders' strategies.

**Key Components**:
- `CopyTradingEngine`: Main copy trading engine
- `TraderProfile`: Trader performance tracking
- `CopyTradeConfig`: Copy trading configuration
- `TraderDiscovery`: Automatic trader discovery
- `TraderAnalysis`: Performance analysis

**Features**:
- Automatic trader discovery
- Performance-based trader selection
- Risk-adjusted position sizing
- Real-time performance tracking
- Trader reputation scoring

**Trader Metrics**:
- Success rate calculation
- Profit/loss tracking
- Trade frequency analysis
- Risk-adjusted returns
- Reputation scoring

**Configuration**:
```rust
let copy_config = CopyTradeConfig {
    min_success_rate: 0.7,
    min_reputation_score: 0.8,
    max_traders_to_follow: 10,
    copy_percentage: 0.1,
    max_copy_amount_sol: 0.01,
    cooldown_between_copies: Duration::from_secs(5),
};
```

### 7. Advanced Bonding Curve Calculations (`bonding_curve.rs`)

**Purpose**: Precise bonding curve mathematics and market analysis.

**Key Components**:
- `BondingCurveCalculator`: Main calculation engine
- `BondingCurveState`: Curve state management
- `AdvancedBondingCurve`: Enhanced calculations with fees
- `BuySimulation`: Buy transaction simulation
- `SellSimulation`: Sell transaction simulation

**Features**:
- Real-time curve state tracking
- Accurate price calculations
- Market cap simulations
- Fee and tax calculations
- Portfolio valuation

**Calculations**:
- Virtual SOL/Token ratios
- Constant product (K) maintenance
- Price impact analysis
- Slippage calculations
- Optimal trade sizing

**Usage Example**:
```rust
let mut calculator = BondingCurveCalculator::new(sol_price_usd);
calculator.initialize_token(&mint, initial_sol_deposit)?;
let simulation = calculator.simulate_buy(&mint, buy_amount_sol)?;
```

## ⚙️ Configuration Guide

### Environment Variables

#### Core Configuration
```bash
# Required
BUYER_PRIVATE_KEY_PATH=your_base58_private_key
GRPC_ENDPOINT=https://your-geyser-endpoint.com
GRPC_AUTH_TOKEN=your_grpc_auth_token

# Optional RPC Configuration
SOLANA_RPC_ENDPOINT=https://your-solana-rpc-endpoint.com
HELIUS_API_KEY=your_helius_api_key

# Trading Parameters
MARKET_CAP_THRESHOLD_USD=8000.0
BUY_AMOUNT_SOL=0.001
```

#### Advanced Features
```bash
# Feature Toggles
ENABLE_JITO=true
ENABLE_COPY_TRADING=false
ENABLE_SCAM_DETECTION=true
ENABLE_SAME_BLOCK_EXECUTION=true
ENABLE_RISK_MANAGEMENT=true

# Risk Management
MAX_SLIPPAGE_PERCENTAGE=20.0
STOP_LOSS_PERCENTAGE=10.0
TAKE_PROFIT_PERCENTAGE=50.0

# Copy Trading
COPY_TRADING_PERCENTAGE=10.0

# Jito Configuration
JITO_TIP_LAMPORTS=10000
```

### Configuration Validation

The bot automatically validates all configuration parameters:

- **Private Key**: Validates base58 format and length
- **URLs**: Ensures proper HTTP/HTTPS format
- **Numeric Values**: Validates positive values and ranges
- **Feature Flags**: Ensures boolean values
- **Dependencies**: Checks required combinations

## 🚀 Setup & Installation

### Prerequisites

- **Rust 1.70+**: Latest stable Rust toolchain
- **Solana CLI**: For wallet management
- **gRPC Endpoint**: Access to Solana Geyser endpoint
- **RPC Endpoint**: Solana RPC access (Helius recommended)

### Installation Steps

1. **Clone Repository**:
   ```bash
   git clone <repository-url>
   cd solana-pumpfun-sniper-bot
   ```

2. **Install Dependencies**:
   ```bash
   cargo build --release
   ```

3. **Configure Environment**:
   ```bash
   cp env.example .env
   # Edit .env with your configuration
   ```

4. **Generate gRPC Code** (if needed):
   ```bash
   protoc --go_out=. --go-grpc_out=. proto/geyser.proto
   ```

5. **Run the Bot**:
   ```bash
   cargo run --release
   ```

### Docker Setup (Optional)

```dockerfile
FROM rust:1.70-slim

WORKDIR /app
COPY . .

RUN cargo build --release

CMD ["./target/release/solana-pumpfun-sniper"]
```

## 📊 Usage Examples

### Basic Sniping

```rust
use solana_pumpfun_sniper::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let mut bot = SniperBot::new(config)?;
    bot.run().await?;
    Ok(())
}
```

### Advanced Configuration

```rust
use solana_pumpfun_sniper::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize components
    let mut risk_manager = RiskManager::new(RiskConfig::default());
    let mut scam_detector = ScamDetector::new();
    let mut copy_engine = CopyTradingEngine::new(CopyTradeConfig::default());
    
    // Create sniper bot with advanced features
    let mut bot = SniperBot::new(config)?;
    
    // Enable advanced features
    bot.enable_risk_management(risk_manager);
    bot.enable_scam_detection(scam_detector);
    bot.enable_copy_trading(copy_engine);
    
    // Run bot
    bot.run().await?;
    Ok(())
}
```

### Custom Risk Management

```rust
use solana_pumpfun_sniper::*;

let risk_config = RiskConfig {
    max_rug_pull_score: 0.2,        // Very conservative
    min_liquidity_sol: 5000.0,      // High liquidity requirement
    min_holder_count: 50,           // Many holders required
    max_slippage_percentage: 10.0,  // Low slippage tolerance
    max_buy_amount_sol: 0.05,       // Small position sizes
    cooldown_period: Duration::from_secs(60), // Longer cooldown
};

let risk_manager = RiskManager::new(risk_config);
```

### Copy Trading Setup

```rust
use solana_pumpfun_sniper::*;

let copy_config = CopyTradeConfig {
    min_success_rate: 0.8,          // High success rate required
    min_reputation_score: 0.9,       // Excellent reputation
    max_traders_to_follow: 5,        // Follow only top traders
    copy_percentage: 0.05,           // Copy 5% of position
    max_copy_amount_sol: 0.005,      // Small copy amounts
    cooldown_between_copies: Duration::from_secs(10),
};

let mut copy_engine = CopyTradingEngine::new(copy_config);

// Add known successful traders
let trader_profile = TraderProfile {
    wallet_address: Pubkey::from_str("...")?,
    success_rate: 0.85,
    total_trades: 200,
    profitable_trades: 170,
    average_profit: 0.12,
    last_activity: Instant::now(),
    reputation_score: 0.92,
};

copy_engine.add_trader(trader_profile.wallet_address, trader_profile)?;
```

## ⚡ Performance Optimization

### Speed Optimizations

1. **Same-Block Execution**:
   - Transactions executed within the same block
   - Pre-fetched block hashes
   - Priority-based queuing

2. **Jito Integration**:
   - MEV protection
   - Dynamic tip calculation
   - Transaction bundling

3. **Connection Optimization**:
   - Multiple gRPC connections
   - Connection pooling
   - Automatic failover

### Memory Optimization

1. **Efficient Caching**:
   - LRU cache for block hashes
   - Thread-safe price caching
   - Bounded connection pools

2. **Resource Management**:
   - Automatic cleanup of old data
   - Memory-efficient data structures
   - Garbage collection optimization

### Network Optimization

1. **Redundant Connections**:
   - Multiple gRPC endpoints
   - Load balancing
   - Health monitoring

2. **Error Recovery**:
   - Automatic retry mechanisms
   - Exponential backoff
   - Circuit breaker patterns

## 🔒 Security Features

### Wallet Security

1. **Private Key Protection**:
   - Environment variable storage
   - No hardcoded keys
   - Secure key validation

2. **Transaction Security**:
   - Signature validation
   - Replay attack prevention
   - Nonce management

### Risk Protection

1. **Scam Detection**:
   - AI-powered analysis
   - Pattern recognition
   - Real-time risk scoring

2. **Risk Management**:
   - Position sizing limits
   - Stop-loss automation
   - Portfolio diversification

### Network Security

1. **Connection Security**:
   - TLS encryption
   - Authentication tokens
   - Connection validation

2. **Data Integrity**:
   - Checksum validation
   - Signature verification
   - Error detection

## 🔧 Troubleshooting

### Common Issues

#### 1. Connection Problems
```bash
# Check gRPC endpoint
curl -v https://your-geyser-endpoint.com

# Verify authentication token
# Check token validity and permissions
```

#### 2. Transaction Failures
```bash
# Check account balance
solana balance <your-wallet-address>

# Verify RPC endpoint
solana config get
```

#### 3. Performance Issues
```bash
# Monitor system resources
htop
iotop

# Check network latency
ping your-geyser-endpoint.com
```

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run --release
```

### Health Checks

Monitor bot health:
```rust
let stats = bot.get_execution_stats().await;
println!("Pending transactions: {}", stats.pending_transactions);
println!("Queue size: {}", stats.queue_size);
println!("Current block: {}", stats.current_block);
```

## 📚 API Reference

### Core Types

#### `Config`
```rust
pub struct Config {
    pub buyer_private_key: String,
    pub grpc_endpoint: String,
    pub grpc_auth_token: String,
    pub solana_rpc_endpoint: String,
    pub market_cap_threshold_usd: f64,
    pub buy_amount_sol: f64,
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
}
```

#### `RiskMetrics`
```rust
pub struct RiskMetrics {
    pub market_cap: f64,
    pub liquidity: f64,
    pub volume_24h: f64,
    pub holder_count: u32,
    pub is_honeypot: bool,
    pub rug_pull_score: f64,
    pub creation_time: Instant,
}
```

#### `ScamAnalysis`
```rust
pub struct ScamAnalysis {
    pub mint: Pubkey,
    pub scam_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub recommendation: ScamRecommendation,
    pub confidence: f64,
    pub analysis_time: Instant,
}
```

### Key Methods

#### `SniperBot::new(config: Config) -> Result<Self>`
Creates a new sniper bot instance with the provided configuration.

#### `SniperBot::run() -> Result<()>`
Starts the sniper bot and begins monitoring for new tokens.

#### `RiskManager::evaluate_token(mint: &Pubkey, metrics: &RiskMetrics) -> Result<bool>`
Evaluates a token for risk and returns whether it's safe to trade.

#### `ScamDetector::analyze_token(metadata: &TokenMetadata, trading_data: &TradingData) -> ScamAnalysis`
Analyzes a token for scam indicators and returns a risk assessment.

#### `CopyTradingEngine::should_copy_trade(trader: &Pubkey, token: &Pubkey, action: &TradeAction, amount_sol: f64) -> Result<bool>`
Determines whether to copy a trade from a specific trader.

#### `SameBlockExecutor::schedule_transaction(transaction: Transaction, signers: &[&T], priority: ExecutionPriority, target_block_offset: u64) -> Result<Signature>`
Schedules a transaction for same-block execution.

## 🎯 Best Practices

### Configuration

1. **Start Conservative**: Begin with high risk thresholds
2. **Monitor Performance**: Track success rates and adjust
3. **Test Thoroughly**: Use small amounts initially
4. **Backup Configuration**: Keep working configurations

### Risk Management

1. **Diversify**: Don't put all funds in one token
2. **Set Limits**: Use stop-loss and take-profit
3. **Monitor Closely**: Watch for unusual patterns
4. **Regular Reviews**: Assess performance regularly

### Performance

1. **Optimize Connections**: Use multiple gRPC endpoints
2. **Monitor Latency**: Track execution times
3. **Balance Load**: Distribute across connections
4. **Regular Maintenance**: Clean up old data

## 📈 Monitoring & Analytics

### Key Metrics

- **Execution Speed**: Average transaction time
- **Success Rate**: Percentage of successful trades
- **Risk Score**: Average risk assessment
- **Profit/Loss**: Trading performance
- **Connection Health**: gRPC endpoint status

### Logging

The bot provides comprehensive logging:

```rust
// Enable detailed logging
RUST_LOG=trace cargo run --release

// Log levels: error, warn, info, debug, trace
```

### Performance Monitoring

```rust
// Get execution statistics
let stats = bot.get_execution_stats().await;

// Get risk metrics
let risk_stats = risk_manager.get_portfolio_risk().await;

// Get copy trading performance
let copy_stats = copy_engine.get_performance_stats().await;
```

## 🔄 Updates & Maintenance

### Regular Updates

1. **Monitor PumpFun Changes**: Watch for program updates
2. **Update Dependencies**: Keep Rust crates current
3. **Review Configuration**: Adjust parameters as needed
4. **Test New Features**: Validate new functionality

### Version Control

```bash
# Check for updates
git pull origin main

# Update dependencies
cargo update

# Rebuild
cargo build --release
```

## 📞 Support & Community

### Getting Help

1. **Documentation**: Check this guide first
2. **Issues**: Report bugs and feature requests
3. **Discussions**: Join community discussions
4. **Discord**: Real-time support and updates

### Contributing

1. **Fork Repository**: Create your own fork
2. **Create Branch**: Work on feature branches
3. **Submit PR**: Pull request with changes
4. **Code Review**: Participate in reviews

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## ⚠️ Disclaimer

This software is for educational and research purposes only. Trading cryptocurrencies involves substantial risk of loss and is not suitable for all investors. The high degree of leverage can work against you as well as for you. Before deciding to trade, you should carefully consider your investment objectives, level of experience, and risk appetite. The possibility exists that you could sustain a loss of some or all of your initial investment and therefore you should not invest money that you cannot afford to lose. You should be aware of all the risks associated with cryptocurrency trading and seek advice from an independent financial advisor if you have any doubts.

---

*Last Updated: December 2024*
*Version: 2.0.0*
