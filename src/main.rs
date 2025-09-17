use solana_pumpfun_sniper::{config::Config, sniper::SniperBot};
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Solana PumpFun Sniper Bot...");

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Create and run sniper bot
    match SniperBot::new(config) {
        Ok(bot) => {
            if let Err(e) = bot.run().await {
                error!("❌ Sniper bot error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("❌ Failed to create sniper bot: {}", e);
            std::process::exit(1);
        }
    }
}
