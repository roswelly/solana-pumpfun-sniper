use crate::constants::*;
use crate::error::{Result, SniperError};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{info, warn, error};

pub struct JitoClient {
    rpc_client: RpcClient,
    tip_account: Pubkey,
    fee_account: Pubkey,
    enabled: bool,
}

impl JitoClient {
    pub fn new(rpc_endpoint: String, enabled: bool) -> Result<Self> {
        let rpc_client = RpcClient::new(rpc_endpoint);
        let tip_account = Pubkey::from_str(JITO_TIP_ACCOUNT)?;
        let fee_account = Pubkey::from_str(JITO_FEE_ACCOUNT)?;

        Ok(Self {
            rpc_client,
            tip_account,
            fee_account,
            enabled,
        })
    }

    pub async fn send_transaction_with_jito<T: Signer>(
        &self,
        transaction: &Transaction,
        signers: &[&T],
        tip_lamports: u64,
    ) -> Result<Signature> {
        if !self.enabled {
            return self.send_regular_transaction(transaction, signers).await;
        }

        // Create Jito tip instruction
        let tip_instruction = self.create_tip_instruction(tip_lamports)?;
        
        // Clone transaction and add tip instruction
        let mut jito_transaction = transaction.clone();
        jito_transaction.message.instructions.push(tip_instruction);

        // Send transaction with Jito
        match self.rpc_client.send_and_confirm_transaction(jito_transaction) {
            Ok(signature) => {
                info!("Transaction sent with Jito tip: {} lamports", tip_lamports);
                Ok(signature)
            }
            Err(e) => {
                warn!("Jito transaction failed, falling back to regular: {}", e);
                self.send_regular_transaction(transaction, signers).await
            }
        }
    }

    async fn send_regular_transaction<T: Signer>(
        &self,
        transaction: &Transaction,
        signers: &[&T],
    ) -> Result<Signature> {
        self.rpc_client
            .send_and_confirm_transaction(transaction)
            .map_err(|e| SniperError::SolanaClient(format!("Regular transaction failed: {}", e)))
    }

    fn create_tip_instruction(&self, tip_lamports: u64) -> Result<Instruction> {
        Ok(Instruction {
            program_id: solana_sdk::system_program::ID,
            accounts: vec![
                AccountMeta::new(self.tip_account, false),
                AccountMeta::new(self.fee_account, false),
            ],
            data: vec![
                2, // Transfer instruction
                0, 0, 0, 0, 0, 0, 0, 0, // Placeholder for amount
            ],
        })
    }

    pub fn calculate_optimal_tip(&self, priority_fee: u64, urgency: UrgencyLevel) -> u64 {
        let base_tip = match urgency {
            UrgencyLevel::Low => 1000,      // 0.000001 SOL
            UrgencyLevel::Medium => 5000,    // 0.000005 SOL
            UrgencyLevel::High => 10000,     // 0.00001 SOL
            UrgencyLevel::Critical => 50000, // 0.00005 SOL
        };

        // Add priority fee to tip
        base_tip + priority_fee
    }

    pub fn is_jito_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct JitoConfig {
    pub enabled: bool,
    pub default_tip_lamports: u64,
    pub max_tip_lamports: u64,
    pub tip_strategy: TipStrategy,
}

#[derive(Debug, Clone)]
pub enum TipStrategy {
    Fixed(u64),
    Dynamic(DynamicTipConfig),
}

#[derive(Debug, Clone)]
pub struct DynamicTipConfig {
    pub base_tip: u64,
    pub network_congestion_multiplier: f64,
    pub urgency_multiplier: f64,
}

impl Default for JitoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_tip_lamports: 10000, // 0.00001 SOL
            max_tip_lamports: 100000,    // 0.0001 SOL
            tip_strategy: TipStrategy::Dynamic(DynamicTipConfig {
                base_tip: 5000,
                network_congestion_multiplier: 1.5,
                urgency_multiplier: 2.0,
            }),
        }
    }
}

pub struct JitoManager {
    config: JitoConfig,
    client: JitoClient,
    network_congestion: f64,
}

impl JitoManager {
    pub fn new(rpc_endpoint: String, config: JitoConfig) -> Result<Self> {
        let client = JitoClient::new(rpc_endpoint, config.enabled)?;
        
        Ok(Self {
            config,
            client,
            network_congestion: 1.0, // Default congestion level
        })
    }

    pub async fn send_priority_transaction<T: Signer>(
        &self,
        transaction: &Transaction,
        signers: &[&T],
        urgency: UrgencyLevel,
    ) -> Result<Signature> {
        let tip_amount = self.calculate_tip_amount(urgency);
        
        self.client
            .send_transaction_with_jito(transaction, signers, tip_amount)
            .await
    }

    fn calculate_tip_amount(&self, urgency: UrgencyLevel) -> u64 {
        match &self.config.tip_strategy {
            TipStrategy::Fixed(amount) => *amount,
            TipStrategy::Dynamic(config) => {
                let base_tip = config.base_tip;
                let congestion_multiplier = self.network_congestion * config.network_congestion_multiplier;
                let urgency_multiplier = match urgency {
                    UrgencyLevel::Low => 1.0,
                    UrgencyLevel::Medium => 1.5,
                    UrgencyLevel::High => 2.0,
                    UrgencyLevel::Critical => 3.0,
                } * config.urgency_multiplier;

                let calculated_tip = (base_tip as f64 * congestion_multiplier * urgency_multiplier) as u64;
                calculated_tip.min(self.config.max_tip_lamports)
            }
        }
    }

    pub fn update_network_congestion(&mut self, congestion_level: f64) {
        self.network_congestion = congestion_level.clamp(0.1, 10.0);
        info!("Updated network congestion level: {:.2}", self.network_congestion);
    }

    pub fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            congestion_level: self.network_congestion,
            jito_enabled: self.client.is_jito_enabled(),
            recommended_tip: self.calculate_tip_amount(UrgencyLevel::Medium),
        }
    }
}

#[derive(Debug)]
pub struct NetworkStats {
    pub congestion_level: f64,
    pub jito_enabled: bool,
    pub recommended_tip: u64,
}

pub struct JitoBundleBuilder {
    transactions: Vec<Transaction>,
    tip_lamports: u64,
}

impl JitoBundleBuilder {
    pub fn new(tip_lamports: u64) -> Self {
        Self {
            transactions: Vec::new(),
            tip_lamports,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn build(self) -> JitoBundle {
        JitoBundle {
            transactions: self.transactions,
            tip_lamports: self.tip_lamports,
        }
    }
}

pub struct JitoBundle {
    pub transactions: Vec<Transaction>,
    pub tip_lamports: u64,
}

impl JitoBundle {
    pub fn send(&self, client: &JitoClient) -> Result<Vec<Signature>> {
        let mut signatures = Vec::new();
        
        for transaction in &self.transactions {
            let signature = client
                .send_transaction_with_jito(transaction, &[], self.tip_lamports)?;
            signatures.push(signature);
        }
        
        Ok(signatures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jito_config() {
        let config = JitoConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_tip_lamports, 10000);
    }

    #[test]
    fn test_tip_calculation() {
        let config = JitoConfig::default();
        let manager = JitoManager::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            config,
        ).unwrap();
        
        let tip = manager.calculate_tip_amount(UrgencyLevel::High);
        assert!(tip > 0);
    }
}
