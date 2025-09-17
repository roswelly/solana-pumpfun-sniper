use crate::error::{Result, SniperError};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationEvent {
    pub token_mint: Pubkey,
    pub migration_time: Instant,
    pub migration_type: MigrationType,
    pub liquidity_migrated: f64,
    pub pump_swap_address: Option<Pubkey>,
    pub creator_address: Pubkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationType {
    Instant,        // New Season 2 instant migration
    Traditional,    // Old migration with fees
    PumpSwap,      // Migration to PumpSwap platform
}

#[derive(Debug, Clone)]
pub struct MigrationDetector {
    migration_events: HashMap<Pubkey, MigrationEvent>,
    pump_swap_program_id: Pubkey,
    migration_threshold: f64,
    last_check: Instant,
}

impl MigrationDetector {
    pub fn new() -> Result<Self> {
        // PumpSwap program ID (needs to be verified)
        let pump_swap_program_id = Pubkey::from_str("PumpSwap1111111111111111111111111111111111")?;
        
        Ok(Self {
            migration_events: HashMap::new(),
            pump_swap_program_id,
            migration_threshold: 0.95, // 95% of bonding curve completed
            last_check: Instant::now(),
        })
    }

    pub fn detect_migration(&mut self, token_mint: &Pubkey, bonding_curve_state: &BondingCurveState) -> Option<MigrationEvent> {
        // Check if token is ready for migration (Season 2 criteria)
        if self.is_ready_for_migration(bonding_curve_state) {
            let migration_event = MigrationEvent {
                token_mint: *token_mint,
                migration_time: Instant::now(),
                migration_type: MigrationType::Instant, // Season 2 instant migration
                liquidity_migrated: bonding_curve_state.real_sol,
                pump_swap_address: self.calculate_pump_swap_address(token_mint),
                creator_address: Pubkey::default(), // Would need to be extracted from token metadata
            };

            self.migration_events.insert(*token_mint, migration_event.clone());
            info!("🚀 Migration detected for token {} - Instant migration to PumpSwap", token_mint);
            
            return Some(migration_event);
        }

        None
    }

    fn is_ready_for_migration(&self, bonding_curve_state: &BondingCurveState) -> bool {
        // Season 2 criteria: Instant migration when bonding curve is complete
        // This is a simplified check - in reality, we'd need to monitor the actual migration events
        
        // Check if bonding curve is nearly complete
        let completion_ratio = bonding_curve_state.real_sol / (bonding_curve_state.virtual_sol * 0.8);
        completion_ratio >= self.migration_threshold
    }

    fn calculate_pump_swap_address(&self, token_mint: &Pubkey) -> Option<Pubkey> {
        // Calculate the PumpSwap address for the token
        // This would be the program-derived address for the token on PumpSwap
        solana_sdk::pubkey::Pubkey::create_program_address(
            &[b"pump_swap", token_mint.as_ref()],
            &self.pump_swap_program_id,
        ).ok()
    }

    pub fn get_migration_status(&self, token_mint: &Pubkey) -> Option<&MigrationEvent> {
        self.migration_events.get(token_mint)
    }

    pub fn is_token_migrated(&self, token_mint: &Pubkey) -> bool {
        self.migration_events.contains_key(token_mint)
    }

    pub fn get_migration_events(&self) -> Vec<&MigrationEvent> {
        self.migration_events.values().collect()
    }

    pub fn cleanup_old_events(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.migration_events.retain(|_, event| {
            now.duration_since(event.migration_time) < max_age
        });
    }
}

#[derive(Debug, Clone)]
pub struct BondingCurveState {
    pub virtual_sol: f64,
    pub virtual_tokens: f64,
    pub real_sol: f64,
    pub real_tokens: f64,
    pub k: f64,
}

impl BondingCurveState {
    pub fn new() -> Self {
        Self {
            virtual_sol: 30.0,
            virtual_tokens: 1_073_000_000.0,
            real_sol: 0.0,
            real_tokens: 0.0,
            k: 30.0 * 1_073_000_000.0,
        }
    }

    pub fn from_initial_deposit(initial_sol: f64) -> Self {
        let virtual_sol = 30.0 + initial_sol;
        let virtual_tokens = 1_073_000_000.0;
        let k = virtual_sol * virtual_tokens;

        Self {
            virtual_sol,
            virtual_tokens,
            real_sol: initial_sol,
            real_tokens: 0.0,
            k,
        }
    }
}

pub struct PumpSwapMonitor {
    migration_detector: MigrationDetector,
    pump_swap_tokens: HashMap<Pubkey, PumpSwapToken>,
}

#[derive(Debug, Clone)]
pub struct PumpSwapToken {
    pub mint: Pubkey,
    pub pump_swap_address: Pubkey,
    pub liquidity: f64,
    pub volume_24h: f64,
    pub price: f64,
    pub last_update: Instant,
}

impl PumpSwapMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            migration_detector: MigrationDetector::new()?,
            pump_swap_tokens: HashMap::new(),
        })
    }

    pub fn monitor_migration(&mut self, token_mint: &Pubkey, bonding_curve_state: &BondingCurveState) -> Option<MigrationEvent> {
        self.migration_detector.detect_migration(token_mint, bonding_curve_state)
    }

    pub fn add_pump_swap_token(&mut self, token: PumpSwapToken) {
        self.pump_swap_tokens.insert(token.mint, token);
        info!("Added PumpSwap token: {}", token.mint);
    }

    pub fn get_pump_swap_token(&self, mint: &Pubkey) -> Option<&PumpSwapToken> {
        self.pump_swap_tokens.get(mint)
    }

    pub fn update_pump_swap_liquidity(&mut self, mint: &Pubkey, new_liquidity: f64) {
        if let Some(token) = self.pump_swap_tokens.get_mut(mint) {
            token.liquidity = new_liquidity;
            token.last_update = Instant::now();
        }
    }

    pub fn get_all_pump_swap_tokens(&self) -> Vec<&PumpSwapToken> {
        self.pump_swap_tokens.values().collect()
    }
}

pub struct CreatorRevenueTracker {
    creator_revenues: HashMap<Pubkey, CreatorRevenue>,
}

#[derive(Debug, Clone)]
pub struct CreatorRevenue {
    pub creator_address: Pubkey,
    pub total_revenue: f64,
    pub tokens_created: u32,
    pub average_revenue_per_token: f64,
    pub last_payout: Instant,
    pub revenue_share_percentage: f64,
}

impl CreatorRevenueTracker {
    pub fn new() -> Self {
        Self {
            creator_revenues: HashMap::new(),
        }
    }

    pub fn track_creator_revenue(&mut self, creator: Pubkey, revenue: f64, token_mint: Pubkey) {
        let creator_revenue = self.creator_revenues.entry(creator).or_insert(CreatorRevenue {
            creator_address: creator,
            total_revenue: 0.0,
            tokens_created: 0,
            average_revenue_per_token: 0.0,
            last_payout: Instant::now(),
            revenue_share_percentage: 0.05, // 5% default revenue share
        });

        creator_revenue.total_revenue += revenue;
        creator_revenue.tokens_created += 1;
        creator_revenue.average_revenue_per_token = creator_revenue.total_revenue / creator_revenue.tokens_created as f64;
        creator_revenue.last_payout = Instant::now();

        info!("Creator {} earned {} SOL from token {}", creator, revenue, token_mint);
    }

    pub fn get_creator_revenue(&self, creator: &Pubkey) -> Option<&CreatorRevenue> {
        self.creator_revenues.get(creator)
    }

    pub fn get_top_creators(&self, limit: usize) -> Vec<&CreatorRevenue> {
        let mut creators: Vec<_> = self.creator_revenues.values().collect();
        creators.sort_by(|a, b| b.total_revenue.partial_cmp(&a.total_revenue).unwrap());
        creators.into_iter().take(limit).collect()
    }
}

pub struct Season2Features {
    migration_monitor: PumpSwapMonitor,
    creator_tracker: CreatorRevenueTracker,
    instant_migration_enabled: bool,
    zero_migration_fees: bool,
}

impl Season2Features {
    pub fn new() -> Result<Self> {
        Ok(Self {
            migration_monitor: PumpSwapMonitor::new()?,
            creator_tracker: CreatorRevenueTracker::new(),
            instant_migration_enabled: true,
            zero_migration_fees: true,
        })
    }

    pub fn process_token_update(&mut self, token_mint: &Pubkey, bonding_curve_state: &BondingCurveState) -> Option<MigrationEvent> {
        // Monitor for Season 2 instant migrations
        if self.instant_migration_enabled {
            return self.migration_monitor.monitor_migration(token_mint, bonding_curve_state);
        }
        None
    }

    pub fn handle_migration_event(&mut self, migration_event: &MigrationEvent) {
        // Track creator revenue from migration
        if migration_event.liquidity_migrated > 0.0 {
            let revenue = migration_event.liquidity_migrated * 0.01; // 1% revenue share
            self.creator_tracker.track_creator_revenue(
                migration_event.creator_address,
                revenue,
                migration_event.token_mint,
            );
        }

        // Add to PumpSwap monitoring
        if let Some(pump_swap_address) = migration_event.pump_swap_address {
            let pump_swap_token = PumpSwapToken {
                mint: migration_event.token_mint,
                pump_swap_address,
                liquidity: migration_event.liquidity_migrated,
                volume_24h: 0.0,
                price: 0.0,
                last_update: Instant::now(),
            };
            self.migration_monitor.add_pump_swap_token(pump_swap_token);
        }
    }

    pub fn get_migration_stats(&self) -> MigrationStats {
        let migration_events = self.migration_monitor.migration_detector.get_migration_events();
        let pump_swap_tokens = self.migration_monitor.get_all_pump_swap_tokens();
        let top_creators = self.creator_tracker.get_top_creators(10);

        MigrationStats {
            total_migrations: migration_events.len(),
            pump_swap_tokens_count: pump_swap_tokens.len(),
            total_liquidity_migrated: pump_swap_tokens.iter().map(|t| t.liquidity).sum(),
            top_creators_count: top_creators.len(),
            instant_migration_enabled: self.instant_migration_enabled,
            zero_migration_fees: self.zero_migration_fees,
        }
    }
}

#[derive(Debug)]
pub struct MigrationStats {
    pub total_migrations: usize,
    pub pump_swap_tokens_count: usize,
    pub total_liquidity_migrated: f64,
    pub top_creators_count: usize,
    pub instant_migration_enabled: bool,
    pub zero_migration_fees: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_detection() {
        let mut detector = MigrationDetector::new().unwrap();
        let bonding_curve = BondingCurveState::from_initial_deposit(25.0); // Near completion
        
        let migration = detector.detect_migration(&Pubkey::new_unique(), &bonding_curve);
        assert!(migration.is_some());
    }

    #[test]
    fn test_creator_revenue_tracking() {
        let mut tracker = CreatorRevenueTracker::new();
        let creator = Pubkey::new_unique();
        
        tracker.track_creator_revenue(creator, 1.0, Pubkey::new_unique());
        let revenue = tracker.get_creator_revenue(&creator).unwrap();
        
        assert_eq!(revenue.total_revenue, 1.0);
        assert_eq!(revenue.tokens_created, 1);
    }
}
