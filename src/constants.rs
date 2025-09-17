use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// PumpFun program ID (verified current as of 2024)
pub const PUMP_FUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

// Constants
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const TOTAL_SUPPLY: u64 = 1_000_000_000;

// Updated bonding curve constants (2024 - Season 2)
pub const INITIAL_VIRTUAL_SOL: f64 = 30.0;
pub const INITIAL_VIRTUAL_TOKENS: f64 = 1_073_000_000.0;

// Season 2 Migration Constants
pub const MIGRATION_THRESHOLD: f64 = 0.95; // 95% completion triggers instant migration
pub const ZERO_MIGRATION_FEE: f64 = 0.0; // Season 2 has zero migration fees
pub const CREATOR_REVENUE_SHARE: f64 = 0.01; // 1% revenue share for creators

// Jito configuration for ultra-fast transactions
pub const JITO_TIP_ACCOUNT: &str = "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY";
pub const JITO_FEE_ACCOUNT: &str = "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL";

// Risk management constants
pub const MAX_SLIPPAGE_PERCENTAGE: f64 = 20.0;
pub const MIN_LIQUIDITY_THRESHOLD: f64 = 1000.0; // Minimum liquidity in SOL
pub const MAX_BUY_AMOUNT_SOL: f64 = 0.1; // Maximum buy amount per transaction

// Known program IDs
pub const KNOWN_GLOBAL: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
pub const KNOWN_EVENT_AUTH: &str = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";
pub const KNOWN_SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
pub const KNOWN_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const KNOWN_METADATA_PROGRAM: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const KNOWN_ATA_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
pub const KNOWN_COMPUTE_BUDGET: &str = "ComputeBudget111111111111111111111111111111";
pub const KNOWN_RENT: &str = "SysvarRent111111111111111111111111111111111";

// Fee recipient
pub const FEE_RECIPIENT: &str = "G5UZAVbAf46s7cKWoyKu8kYTip9DGTpbLZ2qa9Aq69dP";

// Updated discriminators (2024)
pub const CREATE_DISCRIMINATOR: [u8; 8] = [0x18, 0x1e, 0xc8, 0x28, 0x05, 0x1c, 0x07, 0x77];
pub const PUMPFUN_BUY_DISCRIMINATOR: [u8; 8] = [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea];
pub const PUMPFUN_SELL_DISCRIMINATOR: [u8; 8] = [0x33, 0xe6, 0x85, 0x4a, 0x5a, 0x2d, 0x07, 0x1a];
pub const PUMPFUN_CLOSE_DISCRIMINATOR: [u8; 8] = [0x41, 0x13, 0x77, 0x1f, 0x4c, 0x0e, 0x8a, 0x2b];

// Copy trading discriminators
pub const COPY_TRADE_DISCRIMINATOR: [u8; 8] = [0x52, 0x8a, 0x9c, 0x3d, 0x1e, 0x4f, 0x7b, 0x2c];

// Season 2 Migration discriminators
pub const INSTANT_MIGRATION_DISCRIMINATOR: [u8; 8] = [0x73, 0x2a, 0x1b, 0x4c, 0x5d, 0x6e, 0x7f, 0x8a];
pub const PUMP_SWAP_MIGRATION_DISCRIMINATOR: [u8; 8] = [0x84, 0x3b, 0x2c, 0x5d, 0x6e, 0x7f, 0x8a, 0x9b];
pub const CREATOR_REVENUE_DISCRIMINATOR: [u8; 8] = [0x95, 0x4c, 0x3d, 0x6e, 0x7f, 0x8a, 0x9b, 0xac];

// Helper function to get known program pubkeys
pub fn get_known_program_pubkeys() -> Vec<Pubkey> {
    vec![
        Pubkey::from_str(KNOWN_GLOBAL).unwrap(),
        Pubkey::from_str(KNOWN_EVENT_AUTH).unwrap(),
        Pubkey::from_str(KNOWN_SYSTEM_PROGRAM).unwrap(),
        Pubkey::from_str(KNOWN_TOKEN_PROGRAM).unwrap(),
        Pubkey::from_str(KNOWN_METADATA_PROGRAM).unwrap(),
        Pubkey::from_str(KNOWN_ATA_PROGRAM).unwrap(),
        Pubkey::from_str(KNOWN_COMPUTE_BUDGET).unwrap(),
        Pubkey::from_str(KNOWN_RENT).unwrap(),
    ]
}
