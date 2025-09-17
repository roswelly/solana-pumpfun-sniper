use crate::{
    config::Config,
    constants::*,
    error::{Result, SniperError},
    geyser::*,
    price_cache::PriceCache,
};
use anyhow::anyhow;
use parking_lot::Mutex;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::sync::Arc;
use std::str::FromStr;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::Request;
use tracing::{error, info, warn};

pub struct SniperBot {
    config: Config,
    price_cache: Arc<PriceCache>,
    rpc_client: RpcClient,
    buyer_keypair: Keypair,
    processing_mutex: Arc<Mutex<()>>,
}

impl SniperBot {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let price_cache = Arc::new(PriceCache::new());
        let rpc_client = RpcClient::new(config.solana_rpc_endpoint.clone());
        
        // Parse private key from base58 string
        let private_key_bytes = bs58::decode(&config.buyer_private_key)
            .into_vec()
            .map_err(|e| SniperError::Base58(e))?;
        
        let buyer_keypair = Keypair::from_bytes(&private_key_bytes)
            .map_err(|e| SniperError::SolanaClient(format!("Invalid private key: {}", e)))?;

        info!("✅ Buyer's Public Key: {}", buyer_keypair.pubkey());

        Ok(Self {
            config,
            price_cache,
            rpc_client,
            buyer_keypair,
            processing_mutex: Arc::new(Mutex::new(())),
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("🚀 Starting sniper bot monitoring...");

        // Start price cache updates
        let price_cache = Arc::clone(&self.price_cache);
        tokio::spawn(async move {
            price_cache.update_price_periodically().await;
        });

        // Wait for initial price fetch
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // Connect to gRPC endpoint
        let channel = Channel::from_shared(self.config.grpc_endpoint.clone())
            .map_err(|e| SniperError::Grpc(tonic::Status::from_error(e)))?
            .connect()
            .await
            .map_err(|e| SniperError::Grpc(tonic::Status::from_error(e)))?;

        let mut client = GeyserClient::new(channel);

        // Create subscription request
        let subscription_request = SubscribeRequest {
            transactions: [(
                "pump_fun_subscription".to_string(),
                SubscribeRequestFilterTransactions {
                    vote: false,
                    failed: false,
                    account_include: vec![PUMP_FUN_PROGRAM_ID.to_string()],
                },
            )]
            .into(),
            transactions_status: [(
                "pump_fun_status".to_string(),
                SubscribeRequestFilterTransactions {
                    vote: false,
                    failed: false,
                    account_include: vec![PUMP_FUN_PROGRAM_ID.to_string()],
                },
            )]
            .into(),
            commitment: CommitmentLevel::Processed as i32,
        };

        info!("🔌 Connecting to Geyser: {}", self.config.grpc_endpoint);
        
        let mut stream = client
            .subscribe(Request::new(subscription_request))
            .await
            .map_err(|e| SniperError::Grpc(e))?
            .into_inner();

        info!("✅ gRPC Connection Established.");
        info!("✅ Subscribed. Waiting for 'create' transactions...");
        info!("🎯 Monitoring for tokens with market cap >= ${:.2}", self.config.market_cap_threshold_usd);

        // Process incoming transactions
        while let Some(response) = stream.message().await.map_err(|e| SniperError::Grpc(e))? {
            if let Some(tx_update) = response.transaction {
                if let Err(e) = self.process_transaction(tx_update).await {
                    error!("Error processing transaction: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn process_transaction(&self, tx_update: TransactionUpdate) -> Result<()> {
        let tx = tx_update.transaction.ok_or_else(|| {
            SniperError::Transaction("Missing transaction in update".to_string())
        })?;

        let message = tx.message.ok_or_else(|| {
            SniperError::Transaction("Missing message in transaction".to_string())
        })?;

        let meta = tx.meta.ok_or_else(|| {
            SniperError::Transaction("Missing meta in transaction".to_string())
        })?;

        // Combine all account keys
        let mut full_account_list = message.account_keys.clone();
        full_account_list.extend_from_slice(&meta.loaded_writable_addresses);
        full_account_list.extend_from_slice(&meta.loaded_readonly_addresses);

        // Find PumpFun program index
        let pump_fun_pk = Pubkey::from_str(PUMP_FUN_PROGRAM_ID)?;
        let pump_fun_program_index = full_account_list
            .iter()
            .position(|key_bytes| {
                Pubkey::try_from(key_bytes.as_slice())
                    .map(|pk| pk == pump_fun_pk)
                    .unwrap_or(false)
            })
            .ok_or_else(|| SniperError::Transaction("PumpFun program not found in accounts".to_string()))?;

        // Process instructions
        for instruction in &message.instructions {
            if instruction.program_id_index as usize == pump_fun_program_index {
                if instruction.data.starts_with(&CREATE_DISCRIMINATOR) {
                    self.handle_create_instruction(instruction, &full_account_list, &meta).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_create_instruction(
        &self,
        instruction: &Instruction,
        full_account_list: &[Vec<u8>],
        meta: &Meta,
    ) -> Result<()> {
        if instruction.accounts.len() < 8 {
            return Ok(());
        }

        // Extract account keys
        let (mint_key, bonding_curve_key, associated_bonding_curve_key, creator_vault_key) = 
            self.extract_account_keys(instruction, full_account_list)?;

        // Calculate initial SOL deposit
        let initial_sol_lamports = self.calculate_initial_sol_deposit(
            instruction,
            full_account_list,
            meta,
            &bonding_curve_key,
        )?;

        if initial_sol_lamports == 0 {
            return Ok(());
        }

        // Calculate market cap
        let sol_price_usd = self.price_cache.get();
        if sol_price_usd <= 0.0 {
            warn!("SOL price not available, skipping transaction");
            return Ok(());
        }

        let sol_deposited_in_sol = initial_sol_lamports as f64 / LAMPORTS_PER_SOL as f64;
        let k = INITIAL_VIRTUAL_SOL * INITIAL_VIRTUAL_TOKENS;
        let virtual_sol_after = INITIAL_VIRTUAL_SOL + sol_deposited_in_sol;
        let virtual_tokens_after = k / virtual_sol_after;
        let current_price_in_sol = virtual_sol_after / virtual_tokens_after;
        let current_price_usd = current_price_in_sol * sol_price_usd;
        let market_cap_usd = current_price_usd * TOTAL_SUPPLY as f64;

        if market_cap_usd >= self.config.market_cap_threshold_usd {
            let _guard = self.processing_mutex.lock();
            
            info!("🎯 TARGET ACQUIRED - Market Cap: ${:.2} | Mint: {}", market_cap_usd, mint_key);
            info!("🚀 Attempting buy transaction...");

            self.execute_buy_transaction(
                &mint_key,
                &bonding_curve_key,
                &associated_bonding_curve_key,
                &creator_vault_key,
                initial_sol_lamports,
            ).await?;
        }

        Ok(())
    }

    fn extract_account_keys(
        &self,
        instruction: &Instruction,
        full_account_list: &[Vec<u8>],
    ) -> Result<(Pubkey, Pubkey, Pubkey, Pubkey)> {
        let known_programs = get_known_program_pubkeys();
        let mut unknown_accounts = Vec::new();
        let mut creator_key = Pubkey::default();
        let mut global_key = Pubkey::default();
        let mut event_authority_key = Pubkey::default();

        // Process accounts
        for (i, account_bytes) in full_account_list.iter().enumerate() {
            let account_pk = Pubkey::try_from(account_bytes.as_slice())
                .map_err(|e| SniperError::Transaction(format!("Invalid account key: {}", e)))?;

            if i == 0 {
                creator_key = account_pk;
            }

            if account_pk == Pubkey::from_str(KNOWN_GLOBAL)? {
                global_key = account_pk;
            } else if account_pk == Pubkey::from_str(KNOWN_EVENT_AUTH)? {
                event_authority_key = account_pk;
            } else if !known_programs.contains(&account_pk) {
                unknown_accounts.push(account_pk);
            }
        }

        // Find mint key (ends with "pump")
        let mint_key = unknown_accounts
            .iter()
            .find(|pk| pk.to_string().ends_with("pump"))
            .copied()
            .unwrap_or_else(|| {
                // Fallback: use first instruction account
                if !instruction.accounts.is_empty() {
                    Pubkey::try_from(full_account_list[instruction.accounts[0] as usize].as_slice())
                        .unwrap_or_default()
                } else {
                    Pubkey::default()
                }
            });

        // Find bonding curve and associated bonding curve keys
        let remaining_accounts: Vec<_> = unknown_accounts
            .into_iter()
            .filter(|pk| *pk != mint_key && *pk != creator_key)
            .collect();

        let bonding_curve_key = if remaining_accounts.len() >= 2 {
            remaining_accounts[0]
        } else if instruction.accounts.len() > 2 {
            Pubkey::try_from(full_account_list[instruction.accounts[2] as usize].as_slice())?
        } else {
            return Err(SniperError::Transaction("Could not find bonding curve key".to_string()));
        };

        let associated_bonding_curve_key = if remaining_accounts.len() >= 2 {
            remaining_accounts[1]
        } else if instruction.accounts.len() > 3 {
            Pubkey::try_from(full_account_list[instruction.accounts[3] as usize].as_slice())?
        } else {
            return Err(SniperError::Transaction("Could not find associated bonding curve key".to_string()));
        };

        // Find creator vault key
        let creator_vault_key = if full_account_list.len() > 7 {
            Pubkey::try_from(full_account_list[7].as_slice())?
        } else {
            return Err(SniperError::Transaction("Could not find creator vault key".to_string()));
        };

        Ok((mint_key, bonding_curve_key, associated_bonding_curve_key, creator_vault_key))
    }

    fn calculate_initial_sol_deposit(
        &self,
        instruction: &Instruction,
        full_account_list: &[Vec<u8>],
        meta: &Meta,
        bonding_curve_key: &Pubkey,
    ) -> Result<u64> {
        let mut initial_sol_lamports = 0u64;
        let creator_key = Pubkey::try_from(full_account_list[0].as_slice())?;

        for inner_instruction in &meta.inner_instructions {
            for inst in &inner_instruction.instructions {
                let prog_key = Pubkey::try_from(full_account_list[inst.program_id_index as usize].as_slice())?;
                
                if prog_key == solana_sdk::system_program::ID {
                    if inst.data.len() >= 8 {
                        let instruction_type = u32::from_le_bytes([
                            inst.data[0], inst.data[1], inst.data[2], inst.data[3]
                        ]);
                        
                        if instruction_type == system_instruction::SystemInstruction::Transfer as u32 {
                            let source_key = Pubkey::try_from(full_account_list[inst.accounts[0] as usize].as_slice())?;
                            let destination_key = Pubkey::try_from(full_account_list[inst.accounts[1] as usize].as_slice())?;
                            let lamports = u64::from_le_bytes([
                                inst.data[4], inst.data[5], inst.data[6], inst.data[7],
                                inst.data[8], inst.data[9], inst.data[10], inst.data[11],
                            ]);

                            if destination_key == *bonding_curve_key && source_key == creator_key {
                                if lamports > initial_sol_lamports {
                                    initial_sol_lamports = lamports;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(initial_sol_lamports)
    }

    async fn execute_buy_transaction(
        &self,
        mint_key: &Pubkey,
        bonding_curve_key: &Pubkey,
        associated_bonding_curve_key: &Pubkey,
        creator_vault_key: &Pubkey,
        initial_sol_lamports: u64,
    ) -> Result<()> {
        // Get buyer's ATA
        let buyer_ata = get_associated_token_address(&self.buyer_keypair.pubkey(), mint_key);

        // Get recent blockhash
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .map_err(|e| SniperError::SolanaClient(format!("Failed to get recent blockhash: {}", e)))?;

        // Calculate buy parameters
        let sol_deposited_in_sol = initial_sol_lamports as f64 / LAMPORTS_PER_SOL as f64;
        let k = INITIAL_VIRTUAL_SOL * INITIAL_VIRTUAL_TOKENS;
        let current_virtual_sol = INITIAL_VIRTUAL_SOL + sol_deposited_in_sol;
        let current_virtual_tokens = k / current_virtual_sol;
        let virtual_sol_after_buy = current_virtual_sol + self.config.buy_amount_sol;
        let virtual_tokens_after_buy = k / virtual_sol_after_buy;
        let tokens_to_buy = current_virtual_tokens - virtual_tokens_after_buy;
        let token_amount_to_buy = (tokens_to_buy * 1_000_000.0) as u64;
        let max_sol_cost_lamports = (self.config.buy_amount_sol * LAMPORTS_PER_SOL as f64 * 1.20) as u64;

        // Build buy instruction data
        let mut buy_instruction_data = PUMPFUN_BUY_DISCRIMINATOR.to_vec();
        buy_instruction_data.extend_from_slice(&token_amount_to_buy.to_le_bytes());
        buy_instruction_data.extend_from_slice(&max_sol_cost_lamports.to_le_bytes());

        // Create transaction
        let mut instructions = vec![
            compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(400_000),
            compute_budget::ComputeBudgetInstruction::set_compute_unit_price(500_000),
        ];

        // Add ATA creation instruction
        instructions.push(spl_associated_token_account::instruction::create_associated_token_account(
            &self.buyer_keypair.pubkey(),
            &self.buyer_keypair.pubkey(),
            mint_key,
            &spl_token::id(),
        ));

        // Add PumpFun buy instruction
        let pump_fun_pk = Pubkey::from_str(PUMP_FUN_PROGRAM_ID)?;
        let global_key = Pubkey::from_str(KNOWN_GLOBAL)?;
        let event_authority_key = Pubkey::from_str(KNOWN_EVENT_AUTH)?;
        let fee_recipient_pk = Pubkey::from_str(FEE_RECIPIENT)?;

        instructions.push(Instruction {
            program_id: pump_fun_pk,
            accounts: vec![
                AccountMeta::new_readonly(global_key, false),
                AccountMeta::new(fee_recipient_pk, false),
                AccountMeta::new(*mint_key, false),
                AccountMeta::new(*bonding_curve_key, false),
                AccountMeta::new(*associated_bonding_curve_key, false),
                AccountMeta::new(buyer_ata, false),
                AccountMeta::new(self.buyer_keypair.pubkey(), true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new(*creator_vault_key, false),
                AccountMeta::new_readonly(event_authority_key, false),
                AccountMeta::new_readonly(pump_fun_pk, false),
            ],
            data: buy_instruction_data,
        });

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.buyer_keypair.pubkey()),
            &[&self.buyer_keypair],
            recent_blockhash,
        );

        // Send transaction
        let signature = self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| SniperError::SolanaClient(format!("Failed to send buy transaction: {}", e)))?;

        info!("✅ Buy Transaction sent! Signature: {}", signature);
        info!("🔍 View on Solscan: https://solscan.io/tx/{}", signature);

        Ok(())
    }
}
