use crate::error::{Result, SniperError};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Signature,
    signer::Signer,
    transaction::Transaction,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

pub struct SameBlockExecutor {
    rpc_client: RpcClient,
    pending_transactions: Arc<RwLock<HashMap<Signature, PendingTransaction>>>,
    block_tracker: BlockTracker,
    execution_queue: ExecutionQueue,
}

#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub signature: Signature,
    pub transaction: Transaction,
    pub target_block: u64,
    pub created_at: Instant,
    pub priority: ExecutionPriority,
    pub retry_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionPriority {
    Low,
    Medium,
    High,
    Critical,
}

pub struct BlockTracker {
    current_block: Arc<RwLock<u64>>,
    block_hash_cache: Arc<RwLock<HashMap<u64, Hash>>>,
    last_update: Arc<RwLock<Instant>>,
}

impl BlockTracker {
    pub fn new() -> Self {
        Self {
            current_block: Arc::new(RwLock::new(0)),
            block_hash_cache: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn update_current_block(&self, rpc_client: &RpcClient) -> Result<u64> {
        match rpc_client.get_slot_with_commitment(CommitmentConfig::processed()).await {
            Ok(slot) => {
                let mut current_block = self.current_block.write().await;
                *current_block = slot;
                
                let mut last_update = self.last_update.write().await;
                *last_update = Instant::now();
                
                info!("Updated current block: {}", slot);
                Ok(slot)
            }
            Err(e) => {
                error!("Failed to get current block: {}", e);
                Err(SniperError::SolanaClient(format!("Failed to get current block: {}", e)))
            }
        }
    }

    pub async fn get_current_block(&self) -> u64 {
        *self.current_block.read().await
    }

    pub async fn get_block_hash(&self, slot: u64, rpc_client: &RpcClient) -> Result<Hash> {
        // Check cache first
        {
            let cache = self.block_hash_cache.read().await;
            if let Some(hash) = cache.get(&slot) {
                return Ok(*hash);
            }
        }

        // Fetch from RPC
        match rpc_client.get_block_hash_with_commitment(slot, CommitmentConfig::processed()).await {
            Ok(Some(hash)) => {
                // Cache the result
                let mut cache = self.block_hash_cache.write().await;
                cache.insert(slot, hash);
                
                // Limit cache size
                if cache.len() > 100 {
                    let oldest_key = *cache.keys().min().unwrap();
                    cache.remove(&oldest_key);
                }
                
                Ok(hash)
            }
            Ok(None) => Err(SniperError::SolanaClient("Block hash not found".to_string())),
            Err(e) => Err(SniperError::SolanaClient(format!("Failed to get block hash: {}", e))),
        }
    }
}

pub struct ExecutionQueue {
    queue: Arc<RwLock<Vec<PendingTransaction>>>,
    max_queue_size: usize,
}

impl ExecutionQueue {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            max_queue_size,
        }
    }

    pub async fn add_transaction(&self, transaction: PendingTransaction) -> Result<()> {
        let mut queue = self.queue.write().await;
        
        if queue.len() >= self.max_queue_size {
            return Err(SniperError::Generic(anyhow::anyhow!("Execution queue is full")));
        }

        // Insert based on priority
        let insert_index = queue.binary_search_by(|tx| {
            tx.priority.cmp(&transaction.priority).reverse()
        }).unwrap_or_else(|i| i);

        queue.insert(insert_index, transaction);
        Ok(())
    }

    pub async fn get_next_transaction(&self) -> Option<PendingTransaction> {
        let mut queue = self.queue.write().await;
        queue.pop()
    }

    pub async fn remove_transaction(&self, signature: &Signature) -> bool {
        let mut queue = self.queue.write().await;
        if let Some(index) = queue.iter().position(|tx| tx.signature == *signature) {
            queue.remove(index);
            true
        } else {
            false
        }
    }

    pub async fn get_queue_size(&self) -> usize {
        self.queue.read().await.len()
    }
}

impl SameBlockExecutor {
    pub fn new(rpc_client: RpcClient) -> Self {
        Self {
            rpc_client,
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            block_tracker: BlockTracker::new(),
            execution_queue: ExecutionQueue::new(1000),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize block tracker
        self.block_tracker.update_current_block(&self.rpc_client).await?;
        
        // Start background tasks
        self.start_block_tracker_task().await;
        self.start_execution_task().await;
        
        Ok(())
    }

    async fn start_block_tracker_task(&self) {
        let block_tracker = self.block_tracker.clone();
        let rpc_client = self.rpc_client.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = block_tracker.update_current_block(&rpc_client).await {
                    error!("Block tracker error: {}", e);
                }
            }
        });
    }

    async fn start_execution_task(&self) {
        let execution_queue = self.execution_queue.clone();
        let pending_transactions = self.pending_transactions.clone();
        let block_tracker = self.block_tracker.clone();
        let rpc_client = self.rpc_client.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(50));
            
            loop {
                interval.tick().await;
                
                let current_block = block_tracker.get_current_block().await;
                
                // Process transactions for current block
                while let Some(mut pending_tx) = execution_queue.get_next_transaction().await {
                    if pending_tx.target_block <= current_block {
                        // Execute transaction
                        match Self::execute_transaction(&rpc_client, &pending_tx).await {
                            Ok(signature) => {
                                info!("Successfully executed transaction in block {}: {}", 
                                      current_block, signature);
                                
                                // Remove from pending
                                pending_transactions.write().await.remove(&pending_tx.signature);
                            }
                            Err(e) => {
                                error!("Failed to execute transaction: {}", e);
                                
                                // Retry logic
                                pending_tx.retry_count += 1;
                                if pending_tx.retry_count < 3 {
                                    // Re-queue with higher priority
                                    pending_tx.priority = ExecutionPriority::High;
                                    if let Err(e) = execution_queue.add_transaction(pending_tx).await {
                                        error!("Failed to re-queue transaction: {}", e);
                                    }
                                } else {
                                    // Remove after max retries
                                    pending_transactions.write().await.remove(&pending_tx.signature);
                                }
                            }
                        }
                    } else {
                        // Re-queue for later
                        if let Err(e) = execution_queue.add_transaction(pending_tx).await {
                            error!("Failed to re-queue transaction: {}", e);
                        }
                        break; // No more transactions to process
                    }
                }
            }
        });
    }

    async fn execute_transaction(
        rpc_client: &RpcClient,
        pending_tx: &PendingTransaction,
    ) -> Result<Signature> {
        rpc_client
            .send_and_confirm_transaction(&pending_tx.transaction)
            .map_err(|e| SniperError::SolanaClient(format!("Transaction execution failed: {}", e)))
    }

    pub async fn schedule_transaction<T: Signer>(
        &self,
        transaction: Transaction,
        signers: &[&T],
        priority: ExecutionPriority,
        target_block_offset: u64,
    ) -> Result<Signature> {
        let current_block = self.block_tracker.get_current_block().await;
        let target_block = current_block + target_block_offset;
        
        // Get fresh blockhash for target block
        let blockhash = self.block_tracker.get_block_hash(target_block, &self.rpc_client).await?;
        
        // Update transaction with fresh blockhash
        let mut updated_transaction = transaction;
        updated_transaction.message.recent_blockhash = blockhash;
        
        // Sign transaction
        updated_transaction.sign(signers, blockhash);
        
        let signature = updated_transaction.signatures[0];
        
        let pending_tx = PendingTransaction {
            signature,
            transaction: updated_transaction,
            target_block,
            created_at: Instant::now(),
            priority,
            retry_count: 0,
        };
        
        // Add to execution queue
        self.execution_queue.add_transaction(pending_tx).await?;
        
        // Track pending transaction
        self.pending_transactions.write().await.insert(signature, pending_tx);
        
        info!("Scheduled transaction for block {}: {}", target_block, signature);
        Ok(signature)
    }

    pub async fn get_execution_stats(&self) -> ExecutionStats {
        let pending_count = self.pending_transactions.read().await.len();
        let queue_size = self.execution_queue.get_queue_size().await;
        let current_block = self.block_tracker.get_current_block().await;
        
        ExecutionStats {
            pending_transactions: pending_count,
            queue_size,
            current_block,
            uptime: Instant::now(), // Would track actual uptime
        }
    }

    pub async fn cancel_transaction(&self, signature: &Signature) -> bool {
        let removed_from_queue = self.execution_queue.remove_transaction(signature).await;
        let removed_from_pending = self.pending_transactions.write().await.remove(signature).is_some();
        
        removed_from_queue || removed_from_pending
    }
}

#[derive(Debug)]
pub struct ExecutionStats {
    pub pending_transactions: usize,
    pub queue_size: usize,
    pub current_block: u64,
    pub uptime: Instant,
}

pub struct SameBlockSniper {
    executor: SameBlockExecutor,
    snipe_config: SnipeConfig,
}

#[derive(Debug, Clone)]
pub struct SnipeConfig {
    pub max_slippage: f64,
    pub max_gas_price: u64,
    pub target_block_offset: u64,
    pub priority: ExecutionPriority,
}

impl Default for SnipeConfig {
    fn default() -> Self {
        Self {
            max_slippage: 0.05, // 5%
            max_gas_price: 1000000, // 0.001 SOL
            target_block_offset: 1, // Next block
            priority: ExecutionPriority::Critical,
        }
    }
}

impl SameBlockSniper {
    pub fn new(rpc_client: RpcClient, snipe_config: SnipeConfig) -> Self {
        Self {
            executor: SameBlockExecutor::new(rpc_client),
            snipe_config,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.executor.initialize().await
    }

    pub async fn snipe_token<T: Signer>(
        &self,
        instructions: Vec<Instruction>,
        signers: &[&T],
        fee_payer: &Pubkey,
    ) -> Result<Signature> {
        let current_block = self.executor.block_tracker.get_current_block().await;
        let target_block = current_block + self.snipe_config.target_block_offset;
        
        // Get blockhash for target block
        let blockhash = self.executor.block_tracker.get_block_hash(target_block, &self.executor.rpc_client).await?;
        
        // Build transaction
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(fee_payer),
            signers,
            blockhash,
        );
        
        // Schedule for same-block execution
        self.executor.schedule_transaction(
            transaction,
            signers,
            self.snipe_config.priority.clone(),
            self.snipe_config.target_block_offset,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_priority() {
        assert!(ExecutionPriority::Critical > ExecutionPriority::High);
        assert!(ExecutionPriority::High > ExecutionPriority::Medium);
        assert!(ExecutionPriority::Medium > ExecutionPriority::Low);
    }

    #[test]
    fn test_snipe_config() {
        let config = SnipeConfig::default();
        assert_eq!(config.max_slippage, 0.05);
        assert_eq!(config.target_block_offset, 1);
    }
}
