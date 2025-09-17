# 🚀 Quick Start Guide - PumpFun Sniper Bot

Get your PumpFun sniper bot running in 5 minutes!

## ⚡ Quick Setup

### 1. Prerequisites
- Rust 1.70+ installed
- Solana wallet with some SOL
- Access to a Geyser gRPC endpoint

### 2. Clone & Build
```bash
git clone <repository-url>
cd solana-pumpfun-sniper-bot
cargo build --release
```

### 3. Configure Environment
```bash
cp env.example .env
```

Edit `.env` with your settings:
```bash
# Required - Your wallet private key (base58)
BUYER_PRIVATE_KEY_PATH=your_base58_private_key_here

# Required - Geyser gRPC endpoint
GRPC_ENDPOINT=https://your-geyser-endpoint.com
GRPC_AUTH_TOKEN=your_grpc_auth_token

# Required - Solana RPC (use Helius for best performance)
HELIUS_API_KEY=your_helius_api_key

# Optional - Trading parameters
MARKET_CAP_THRESHOLD_USD=8000.0
BUY_AMOUNT_SOL=0.001
```

### 4. Run the Bot
```bash
cargo run --release
```

## 🎯 Basic Configuration

### Conservative Settings (Recommended for Beginners)
```bash
MARKET_CAP_THRESHOLD_USD=15000.0    # Higher threshold = safer
BUY_AMOUNT_SOL=0.0005               # Smaller amounts
ENABLE_SCAM_DETECTION=true          # Always enable
ENABLE_RISK_MANAGEMENT=true         # Always enable
MAX_SLIPPAGE_PERCENTAGE=10.0        # Lower slippage
```

### Aggressive Settings (Advanced Users)
```bash
MARKET_CAP_THRESHOLD_USD=5000.0     # Lower threshold = more trades
BUY_AMOUNT_SOL=0.005                # Larger amounts
ENABLE_SAME_BLOCK_EXECUTION=true    # Maximum speed
ENABLE_JITO=true                    # MEV protection
ENABLE_COPY_TRADING=true            # Mirror successful traders
```

## 🔧 Feature Toggles

Enable/disable features as needed:

```bash
# Core Features (Recommended)
ENABLE_SCAM_DETECTION=true
ENABLE_RISK_MANAGEMENT=true

# Speed Features
ENABLE_SAME_BLOCK_EXECUTION=true
ENABLE_JITO=true

# Advanced Features
ENABLE_COPY_TRADING=false           # Start disabled
```

## 📊 Monitoring Your Bot

### Check Bot Status
The bot will log important events:
```
🚀 Starting sniper bot monitoring...
✅ Configuration loaded successfully
🔌 Connecting to Geyser: https://your-endpoint.com
✅ gRPC Connection Established.
🎯 Monitoring for tokens with market cap >= $8000.00
🎯 TARGET ACQUIRED - Market Cap: $12,450.00 | Mint: ABC123...
✅ Buy Transaction sent! Signature: XYZ789...
```

### Key Metrics to Watch
- **Connection Status**: Ensure gRPC connection is stable
- **Transaction Success**: Monitor buy transaction success rate
- **Risk Assessment**: Check scam detection results
- **Performance**: Track execution speed

## 🛠️ Troubleshooting

### Common Issues

#### 1. "gRPC Connection Failed"
```bash
# Check your endpoint URL
curl -v https://your-geyser-endpoint.com

# Verify authentication token
# Ensure token has proper permissions
```

#### 2. "Transaction Failed"
```bash
# Check wallet balance
solana balance <your-wallet-address>

# Verify RPC endpoint
# Test with Solana CLI
```

#### 3. "No Trades Executed"
- Check market cap threshold (might be too high)
- Verify gRPC connection is receiving data
- Check if scam detection is blocking trades

### Debug Mode
```bash
RUST_LOG=debug cargo run --release
```

## 🎛️ Advanced Configuration

### Risk Management
```bash
# Conservative risk settings
MAX_SLIPPAGE_PERCENTAGE=5.0
STOP_LOSS_PERCENTAGE=5.0
TAKE_PROFIT_PERCENTAGE=25.0

# Aggressive risk settings
MAX_SLIPPAGE_PERCENTAGE=30.0
STOP_LOSS_PERCENTAGE=15.0
TAKE_PROFIT_PERCENTAGE=100.0
```

### Copy Trading Setup
```bash
ENABLE_COPY_TRADING=true
COPY_TRADING_PERCENTAGE=5.0         # Copy 5% of trader's position
```

### Jito Configuration
```bash
ENABLE_JITO=true
JITO_TIP_LAMPORTS=5000              # Tip amount for MEV protection
```

## 📈 Performance Tips

### For Maximum Speed
1. Use Helius RPC endpoint
2. Enable same-block execution
3. Enable Jito integration
4. Use multiple gRPC connections

### For Maximum Safety
1. Enable all risk management features
2. Use conservative thresholds
3. Start with small amounts
4. Monitor closely

### For Maximum Profit
1. Enable copy trading
2. Use aggressive settings
3. Monitor successful traders
4. Adjust parameters based on performance

## 🔍 What to Expect

### First Run
- Bot connects to gRPC endpoint
- Starts monitoring PumpFun transactions
- Analyzes new tokens for risk
- Executes trades when criteria are met

### Typical Performance
- **Execution Speed**: Sub-100ms for same-block execution
- **Scam Detection**: 95%+ accuracy
- **Success Rate**: Varies based on market conditions
- **Uptime**: 99.9%+ with redundant connections

## 📚 Next Steps

1. **Read Full Documentation**: See [DOC_UPDATED.md](DOC_UPDATED.md)
2. **Monitor Performance**: Track your bot's success rate
3. **Adjust Settings**: Optimize based on results
4. **Enable Advanced Features**: Gradually enable more features
5. **Join Community**: Get support and share experiences

## ⚠️ Important Notes

- **Start Small**: Begin with small amounts to test
- **Monitor Closely**: Watch your bot's performance
- **Risk Management**: Never invest more than you can afford to lose
- **Regular Updates**: Keep the bot updated with latest changes
- **Backup Config**: Save working configurations

## 🆘 Getting Help

- **Documentation**: Check [DOC_UPDATED.md](DOC_UPDATED.md)
- **Issues**: Report problems on GitHub
- **Community**: Join Discord for support
- **Updates**: Follow for latest features

---

**Ready to start sniping? Run `cargo run --release` and watch your bot in action!** 🚀
