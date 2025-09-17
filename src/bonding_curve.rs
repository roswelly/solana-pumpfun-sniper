use crate::constants::*;
use crate::error::{Result, SniperError};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct BondingCurveState {
    pub virtual_sol: f64,
    pub virtual_tokens: f64,
    pub real_sol: f64,
    pub real_tokens: f64,
    pub k: f64, // Constant product
}

impl BondingCurveState {
    pub fn new() -> Self {
        Self {
            virtual_sol: INITIAL_VIRTUAL_SOL,
            virtual_tokens: INITIAL_VIRTUAL_TOKENS,
            real_sol: 0.0,
            real_tokens: 0.0,
            k: INITIAL_VIRTUAL_SOL * INITIAL_VIRTUAL_TOKENS,
        }
    }

    pub fn from_initial_deposit(initial_sol: f64) -> Self {
        let virtual_sol = INITIAL_VIRTUAL_SOL + initial_sol;
        let virtual_tokens = INITIAL_VIRTUAL_TOKENS;
        let k = virtual_sol * virtual_tokens;

        Self {
            virtual_sol,
            virtual_tokens,
            real_sol: initial_sol,
            real_tokens: 0.0,
            k,
        }
    }

    pub fn get_current_price(&self) -> f64 {
        if self.virtual_tokens == 0.0 {
            return 0.0;
        }
        self.virtual_sol / self.virtual_tokens
    }

    pub fn get_market_cap(&self, sol_price_usd: f64) -> f64 {
        let price_per_token = self.get_current_price();
        price_per_token * sol_price_usd * TOTAL_SUPPLY as f64
    }

    pub fn calculate_buy_output(&self, sol_input: f64) -> (f64, f64) {
        let new_virtual_sol = self.virtual_sol + sol_input;
        let new_virtual_tokens = self.k / new_virtual_sol;
        let tokens_output = self.virtual_tokens - new_virtual_tokens;
        let sol_output = sol_input; // For simplicity, assuming 1:1 for now

        (tokens_output, sol_output)
    }

    pub fn calculate_sell_output(&self, tokens_input: f64) -> (f64, f64) {
        let new_virtual_tokens = self.virtual_tokens + tokens_input;
        let new_virtual_sol = self.k / new_virtual_tokens;
        let sol_output = self.virtual_sol - new_virtual_sol;
        let tokens_output = tokens_input; // For simplicity

        (sol_output, tokens_output)
    }

    pub fn apply_buy(&mut self, sol_input: f64) -> f64 {
        let (tokens_output, _) = self.calculate_buy_output(sol_input);
        self.virtual_sol += sol_input;
        self.virtual_tokens -= tokens_output;
        tokens_output
    }

    pub fn apply_sell(&mut self, tokens_input: f64) -> f64 {
        let (sol_output, _) = self.calculate_sell_output(tokens_input);
        self.virtual_tokens += tokens_input;
        self.virtual_sol -= sol_output;
        sol_output
    }
}

pub struct BondingCurveCalculator {
    curves: HashMap<Pubkey, BondingCurveState>,
    sol_price_usd: f64,
}

impl BondingCurveCalculator {
    pub fn new(sol_price_usd: f64) -> Self {
        Self {
            curves: HashMap::new(),
            sol_price_usd,
        }
    }

    pub fn update_sol_price(&mut self, sol_price_usd: f64) {
        self.sol_price_usd = sol_price_usd;
        info!("Updated SOL price: ${:.2}", sol_price_usd);
    }

    pub fn initialize_token(&mut self, mint: &Pubkey, initial_sol_deposit: f64) -> Result<BondingCurveState> {
        let curve = BondingCurveState::from_initial_deposit(initial_sol_deposit);
        self.curves.insert(*mint, curve.clone());
        
        let market_cap = curve.get_market_cap(self.sol_price_usd);
        info!("Initialized token {} with market cap: ${:.2}", mint, market_cap);
        
        Ok(curve)
    }

    pub fn get_token_state(&self, mint: &Pubkey) -> Option<&BondingCurveState> {
        self.curves.get(mint)
    }

    pub fn calculate_optimal_buy_amount(&self, mint: &Pubkey, target_market_cap: f64) -> Result<f64> {
        let curve = self.curves.get(mint)
            .ok_or_else(|| SniperError::Generic(anyhow::anyhow!("Token not found: {}", mint)))?;

        let current_market_cap = curve.get_market_cap(self.sol_price_usd);
        
        if current_market_cap >= target_market_cap {
            return Ok(0.0); // Already at target
        }

        // Calculate how much SOL is needed to reach target market cap
        let target_price_per_token = target_market_cap / (TOTAL_SUPPLY as f64 * self.sol_price_usd);
        let current_price_per_token = curve.get_current_price();
        
        if target_price_per_token <= current_price_per_token {
            return Ok(0.0);
        }

        // Use binary search to find optimal buy amount
        let mut low = 0.0;
        let mut high = 1.0; // Start with 1 SOL max
        let mut best_amount = 0.0;

        for _ in 0..100 { // Max 100 iterations
            let mid = (low + high) / 2.0;
            let (tokens_output, _) = curve.calculate_buy_output(mid);
            let new_virtual_sol = curve.virtual_sol + mid;
            let new_virtual_tokens = curve.virtual_tokens - tokens_output;
            let new_price_per_token = new_virtual_sol / new_virtual_tokens;
            let new_market_cap = new_price_per_token * self.sol_price_usd * TOTAL_SUPPLY as f64;

            if new_market_cap >= target_market_cap {
                best_amount = mid;
                high = mid;
            } else {
                low = mid;
            }

            if (high - low) < 0.0001 {
                break;
            }
        }

        Ok(best_amount)
    }

    pub fn simulate_buy(&self, mint: &Pubkey, sol_amount: f64) -> Result<BuySimulation> {
        let curve = self.curves.get(mint)
            .ok_or_else(|| SniperError::Generic(anyhow::anyhow!("Token not found: {}", mint)))?;

        let (tokens_output, _) = curve.calculate_buy_output(sol_amount);
        let new_virtual_sol = curve.virtual_sol + sol_amount;
        let new_virtual_tokens = curve.virtual_tokens - tokens_output;
        let new_price_per_token = new_virtual_sol / new_virtual_tokens;
        let new_market_cap = new_price_per_token * self.sol_price_usd * TOTAL_SUPPLY as f64;

        Ok(BuySimulation {
            tokens_received: tokens_output,
            new_price_per_token,
            new_market_cap,
            price_impact: (new_price_per_token - curve.get_current_price()) / curve.get_current_price(),
            slippage: 0.0, // Would need to calculate based on order book
        })
    }

    pub fn simulate_sell(&self, mint: &Pubkey, tokens_amount: f64) -> Result<SellSimulation> {
        let curve = self.curves.get(mint)
            .ok_or_else(|| SniperError::Generic(anyhow::anyhow!("Token not found: {}", mint)))?;

        let (sol_output, _) = curve.calculate_sell_output(tokens_amount);
        let new_virtual_tokens = curve.virtual_tokens + tokens_amount;
        let new_virtual_sol = curve.virtual_sol - sol_output;
        let new_price_per_token = new_virtual_sol / new_virtual_tokens;
        let new_market_cap = new_price_per_token * self.sol_price_usd * TOTAL_SUPPLY as f64;

        Ok(SellSimulation {
            sol_received: sol_output,
            new_price_per_token,
            new_market_cap,
            price_impact: (curve.get_current_price() - new_price_per_token) / curve.get_current_price(),
            slippage: 0.0, // Would need to calculate based on order book
        })
    }

    pub fn get_all_tokens(&self) -> Vec<(Pubkey, BondingCurveState)> {
        self.curves.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    pub fn calculate_portfolio_value(&self, holdings: &HashMap<Pubkey, f64>) -> f64 {
        let mut total_value = 0.0;

        for (mint, token_amount) in holdings {
            if let Some(curve) = self.curves.get(mint) {
                let price_per_token = curve.get_current_price();
                let value_usd = price_per_token * self.sol_price_usd * token_amount;
                total_value += value_usd;
            }
        }

        total_value
    }
}

#[derive(Debug, Clone)]
pub struct BuySimulation {
    pub tokens_received: f64,
    pub new_price_per_token: f64,
    pub new_market_cap: f64,
    pub price_impact: f64,
    pub slippage: f64,
}

#[derive(Debug, Clone)]
pub struct SellSimulation {
    pub sol_received: f64,
    pub new_price_per_token: f64,
    pub new_market_cap: f64,
    pub price_impact: f64,
    pub slippage: f64,
}

pub struct AdvancedBondingCurve {
    calculator: BondingCurveCalculator,
    fee_rate: f64,
    tax_rate: f64,
}

impl AdvancedBondingCurve {
    pub fn new(sol_price_usd: f64, fee_rate: f64, tax_rate: f64) -> Self {
        Self {
            calculator: BondingCurveCalculator::new(sol_price_usd),
            fee_rate,
            tax_rate,
        }
    }

    pub fn calculate_buy_with_fees(&self, mint: &Pubkey, sol_amount: f64) -> Result<BuySimulationWithFees> {
        let base_simulation = self.calculator.simulate_buy(mint, sol_amount)?;
        
        let fee_amount = sol_amount * self.fee_rate;
        let tax_amount = base_simulation.tokens_received * self.tax_rate;
        let net_tokens = base_simulation.tokens_received - tax_amount;
        let net_sol_cost = sol_amount + fee_amount;

        Ok(BuySimulationWithFees {
            base_simulation,
            fee_amount,
            tax_amount,
            net_tokens,
            net_sol_cost,
            total_cost_usd: net_sol_cost * self.calculator.sol_price_usd,
        })
    }

    pub fn calculate_sell_with_fees(&self, mint: &Pubkey, tokens_amount: f64) -> Result<SellSimulationWithFees> {
        let base_simulation = self.calculator.simulate_sell(mint, tokens_amount)?;
        
        let fee_amount = base_simulation.sol_received * self.fee_rate;
        let tax_amount = tokens_amount * self.tax_rate;
        let net_tokens_sold = tokens_amount - tax_amount;
        let net_sol_received = base_simulation.sol_received - fee_amount;

        Ok(SellSimulationWithFees {
            base_simulation,
            fee_amount,
            tax_amount,
            net_tokens_sold,
            net_sol_received,
            net_value_usd: net_sol_received * self.calculator.sol_price_usd,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BuySimulationWithFees {
    pub base_simulation: BuySimulation,
    pub fee_amount: f64,
    pub tax_amount: f64,
    pub net_tokens: f64,
    pub net_sol_cost: f64,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone)]
pub struct SellSimulationWithFees {
    pub base_simulation: SellSimulation,
    pub fee_amount: f64,
    pub tax_amount: f64,
    pub net_tokens_sold: f64,
    pub net_sol_received: f64,
    pub net_value_usd: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonding_curve_basic() {
        let curve = BondingCurveState::new();
        assert_eq!(curve.virtual_sol, INITIAL_VIRTUAL_SOL);
        assert_eq!(curve.virtual_tokens, INITIAL_VIRTUAL_TOKENS);
    }

    #[test]
    fn test_buy_calculation() {
        let curve = BondingCurveState::new();
        let (tokens, _) = curve.calculate_buy_output(1.0);
        assert!(tokens > 0.0);
    }

    #[test]
    fn test_market_cap_calculation() {
        let curve = BondingCurveState::new();
        let market_cap = curve.get_market_cap(100.0); // $100 SOL
        assert!(market_cap > 0.0);
    }

    #[test]
    fn test_calculator() {
        let mut calculator = BondingCurveCalculator::new(100.0);
        let mint = Pubkey::new_unique();
        let curve = calculator.initialize_token(&mint, 1.0).unwrap();
        assert!(curve.get_market_cap(100.0) > 0.0);
    }
}
