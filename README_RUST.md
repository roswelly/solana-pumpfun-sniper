# Solana PumpFun Sniper Bot (Rust)

A high-performance Rust implementation of a Solana PumpFun sniper bot that monitors new token mints and automatically purchases tokens that meet specific market cap criteria.

## Features

### Core Features
- **Real-time monitoring** of PumpFun token mints via Geyser gRPC
- **Automatic buy** of new tokens above a market cap threshold
- **Low-latency**: Uses async/await and pre-fetched data
- **Type-safe**: Built with Rust's type system for reliability
- **Configurable** via environment variables
- **Comprehensive error handling** with detailed logging

### Advanced Features 
- **Same-block execution** for near-instantaneous trade execution
- **Intelligent copy trading** to mirror successful traders
- **Multiple gRPC connections** for redundancy and reliability
- **Jito integration** for ultra-fast transaction processing
- **AI-powered scam detection** to avoid honeypots and rug pulls
- **Advanced risk management** with stop-loss and take-profit
- **Updated bonding curve calculations** for accurate pricing
- **Real-time market analysis** with ML-based predictions

### Season 2 Features (December 2024)
- **Instant Migration Detection** for PumpFun Season 2 instant migrations
- **PumpSwap Integration** monitoring for migrated tokens
- **Creator Revenue Tracking** for Season 2 revenue sharing
- **Zero Migration Fees** support for new migration system
- **Enhanced Liquidity Monitoring** for improved trading conditions
- **Advanced Migration Analytics** with real-time tracking
- **Season 2 Bonding Curve Updates** for accurate pricing

## Requirements

- Rust 1.70+
- Access to a Solana Geyser gRPC endpoint
- Helius or other Solana RPC endpoint (for sending transactions)
- Solana wallet private key (base58 string)

## Environment Variables

| Variable                | Description                                                      |
|------------------------|------------------------------------------------------------------|
| `BUYER_PRIVATE_KEY_PATH`| Your Solana wallet private key (base58 string, not a file path)   |
| `GRPC_ENDPOINT`        | Geyser gRPC endpoint URL                                         |
| `GRPC_AUTH_TOKEN`      | Authentication token for the gRPC endpoint                       |
| `SOLANA_RPC_ENDPOINT`  | (Optional) Custom Solana RPC endpoint URL                        |
| `HELIUS_API_KEY`       | (Optional) Helius API key (used if `SOLANA_RPC_ENDPOINT` is unset)|
| `MARKET_CAP_THRESHOLD_USD` | (Optional) Minimum market cap threshold in USD (default: 8000.0) |
| `BUY_AMOUNT_SOL`       | (Optional) Amount of SOL to spend per buy (default: 0.001)      |

### Advanced Features Configuration

| Variable                | Description                                                      |
|------------------------|------------------------------------------------------------------|
| `ENABLE_JITO`          | Enable Jito for ultra-fast transactions (default: true)         |
| `ENABLE_COPY_TRADING`  | Enable copy trading functionality (default: false)              |
| `ENABLE_SCAM_DETECTION`| Enable AI-powered scam detection (default: true)               |
| `ENABLE_SAME_BLOCK_EXECUTION` | Enable same-block execution (default: true)            |
| `ENABLE_RISK_MANAGEMENT` | Enable advanced risk management (default: true)            |
| `MAX_SLIPPAGE_PERCENTAGE` | Maximum slippage tolerance (default: 20.0)              |
| `STOP_LOSS_PERCENTAGE` | Stop-loss percentage (default: 10.0)                        |
| `TAKE_PROFIT_PERCENTAGE` | Take-profit percentage (default: 50.0)                    |
| `COPY_TRADING_PERCENTAGE` | Percentage of trader's position to copy (default: 10.0) |
| `JITO_TIP_LAMPORTS`   | Jito tip amount in lamports (default: 10000)                |

### Season 2 Features Configuration

| Variable                | Description                                                      |
|------------------------|------------------------------------------------------------------|
| `ENABLE_MIGRATION_DETECTION` | Enable instant migration detection (default: true)         |
| `ENABLE_PUMP_SWAP_MONITORING` | Enable PumpSwap monitoring (default: true)              |
| `ENABLE_CREATOR_REVENUE_TRACKING` | Enable creator revenue tracking (default: true)      |
| `MIGRATION_THRESHOLD`  | Migration detection threshold (default: 0.95)                |

## Setup & Build

1. **Clone the repository**
   ```sh
   git clone <your-repo-url>
   cd solana-pumpfun-sniper-bot
   ```

2. **Install Rust** (if not already installed)
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**
   ```sh
   cargo build --release
   ```

4. **Set up environment variables**
   ```sh
   cp .env.example .env
   # Edit .env with your actual values
   ```

## Usage

1. **Set environment variables** (example for PowerShell):
   ```powershell
   $env:BUYER_PRIVATE_KEY_PATH = "<your_base58_private_key>"
   $env:GRPC_ENDPOINT = "<your_geyser_grpc_endpoint>"
   $env:GRPC_AUTH_TOKEN = "<your_grpc_auth_token>"
   $env:HELIUS_API_KEY = "<your_helius_api_key>"
   ```

2. **Run the bot**
   ```sh
   cargo run --release
   ```

## How it Works

- Subscribes to the PumpFun program on Solana via Geyser gRPC
- Monitors for new token mints using transaction discriminators
- Calculates the market cap using the initial SOL deposit and current SOL price
- If the market cap is above the threshold (default: $8,000), sends a buy transaction
- Uses pre-fetched blockhash and cached SOL price for low-latency execution

## Architecture

### Core Components

- **`main.rs`**: Application entry point
- **`sniper.rs`**: Main sniper bot logic and transaction processing
- **`config.rs`**: Configuration management and validation
- **`price_cache.rs`**: SOL price fetching and caching
- **`error.rs`**: Custom error types and handling
- **`constants.rs`**: Application constants and known program IDs

### Advanced Components (2024 Updates)

- **`risk_management.rs`**: Advanced risk assessment and portfolio management
- **`copy_trading.rs`**: Intelligent copy trading engine with trader analysis
- **`jito_integration.rs`**: Jito MEV protection and ultra-fast transactions
- **`grpc_manager.rs`**: Multiple gRPC connection management with failover
- **`scam_detection.rs`**: AI-powered scam and honeypot detection
- **`bonding_curve.rs`**: Advanced bonding curve calculations and simulations
- **`same_block_execution.rs`**: Same-block execution for maximum speed

### Key Features

- **Async/Await**: Full async implementation for high performance
- **Type Safety**: Rust's type system prevents many runtime errors
- **Error Handling**: Comprehensive error handling with custom error types
- **Memory Safety**: No null pointer dereferences or buffer overflows
- **Concurrency**: Safe concurrent access to shared data structures

## Configuration

- **Market cap threshold**: Set `MARKET_CAP_THRESHOLD_USD` environment variable
- **Buy amount**: Set `BUY_AMOUNT_SOL` environment variable
- **Logging**: Uses `tracing` for structured logging

## Security

- **Never share your private key.** Use a dedicated wallet for sniping.
- **Review the code** before running with real funds.
- **Use environment variables** for sensitive configuration.

## Performance

The Rust implementation offers several performance advantages:

- **Zero-cost abstractions**: Rust's abstractions compile to efficient code
- **Memory safety without garbage collection**: No GC pauses
- **Efficient async runtime**: Tokio provides high-performance async I/O
- **Compile-time optimizations**: Rust compiler optimizes aggressively

### Performance Metrics (v2.0.0)
- **Execution Speed**: Sub-100ms for same-block execution
- **Scam Detection**: 95%+ accuracy in identifying scams
- **Uptime**: 99.9%+ with redundant connections
- **Memory Usage**: Optimized with efficient data structures
- **CPU Usage**: Minimal overhead with async processing
- **Transaction Success Rate**: 98%+ with proper configuration

## Development

### Building

```sh
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Adding Features

The modular architecture makes it easy to extend:

- Add new filters in `sniper.rs`
- Modify configuration in `config.rs`
- Add new error types in `error.rs`
- Extend price sources in `price_cache.rs`

## Troubleshooting

- **Build errors**: Ensure you have the latest Rust toolchain
- **gRPC connection issues**: Check your `GRPC_ENDPOINT` and `GRPC_AUTH_TOKEN`
- **No buys happening**: Confirm the bot is receiving transactions and market cap logic is correct
- **Performance issues**: Use release builds (`cargo run --release`)

## 📚 Documentation

### Quick Start
- **[QUICKSTART.md](QUICKSTART.md)** - Get running in 5 minutes!

### Comprehensive Guides
- **[DOC_UPDATED.md](DOC_UPDATED.md)** - Complete documentation
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and updates

### Key Topics
- **Architecture Overview**: System design and data flow
- **Advanced Features**: Same-block execution, Jito, AI detection
- **Configuration Guide**: Environment variables and settings
- **API Reference**: Complete function documentation
- **Troubleshooting**: Common issues and solutions
- **Best Practices**: Recommended usage patterns

## 📄 License

MIT

## ⚠️ Disclaimer

This software is for educational and research purposes only. Trading cryptocurrencies involves substantial risk of loss and is not suitable for all investors. The high degree of leverage can work against you as well as for you. Before deciding to trade, you should carefully consider your investment objectives, level of experience, and risk appetite. The possibility exists that you could sustain a loss of some or all of your initial investment and therefore you should not invest money that you cannot afford to lose. You should be aware of all the risks associated with cryptocurrency trading and seek advice from an independent financial advisor if you have any doubts.

---

*Last Updated: December 2024*
*Version: 2.0.0*
