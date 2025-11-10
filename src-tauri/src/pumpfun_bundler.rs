use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signer}, system_instruction, transaction::Transaction, commitment_config::CommitmentLevel};
use solana_client::rpc_client::RpcClient;
use solana_client::client_error::ClientError;
use tokio::sync::Mutex;
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use argon2::{Argon2, password_hash::SaltString};
use rand::rngs::OsRng;
use rand::RngCore;
use thiserror::Error;
use chrono::{DateTime, Utc};
use reqwest::Client;

/// Enhanced error types for Pumpfun and Bundler operations
#[derive(Error, Debug)]
pub enum PumpfunBundlerError {
    #[error("Bundle construction failed: {0}")]
    BundleConstruction(String),
    #[error("Jito bundler API error: {0}")]
    JitoApi(String),
    #[error("Pumpfun program error: {0}")]
    PumpfunProgram(String),
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
    #[error("Transaction simulation failed: {0}")]
    Simulation(String),
    #[error("Bundle submission failed: {0}")]
    BundleSubmission(String),
    #[error("MEV protection failed: {0}")]
    MevProtection(String),
    #[error("Launch+Snipe bundle failed: {0}")]
    LaunchSnipeFailed(String),
    #[error("Token sniping failed: {0}")]
    SnipeFailed(String),
}

// Enhanced Jito bundler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoConfig {
    pub enabled: bool,
    pub bundler_url: String,
    pub max_tip_lamports: u64,
    pub min_tip_lamports: u64,
    pub private_rpc_url: Option<String>,
    pub block_engine_url: String,
}

impl Default for JitoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bundler_url: "https://mainnet.block-engine.jito.wtf/api/v1".to_string(),
            max_tip_lamports: 5_000_000, // 0.005 SOL max tip
            min_tip_lamports: 100_000,   // 0.0001 SOL min tip
            private_rpc_url: None,
            block_engine_url: "https://mainnet.block-engine.jito.wtf".to_string(),
        }
    }
}

// Bundle transaction structure for Jito
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleTransaction {
    pub transaction: Vec<u8>,
    pub signer_public_keys: Vec<String>,
    pub version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSubmission {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
    pub params: Vec<BundleTransaction>,
    pub tip: u64,
}

// MEV protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevProtection {
    pub use_private_rpc: bool,
    pub randomize_timing: bool,
    pub hide_transaction_size: bool,
    pub flashloan_protection: bool,
    pub simulate_before_submit: bool,
    pub max_retry_attempts: u32,
}

impl Default for MevProtection {
    fn default() -> Self {
        Self {
            use_private_rpc: true,
            randomize_timing: true,
            hide_transaction_size: true,
            flashloan_protection: true,
            simulate_before_submit: true,
            max_retry_attempts: 3,
        }
    }
}

// Enhanced pump.fun token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpfunTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
    pub initial_liquidity_sol: f64,
    pub slippage_bps: u16,
    pub dev_fee_bps: u8,
    pub buy_amount_sol: f64, // Amount to snipe immediately
    pub auto_snipe: bool,    // Enable auto sniping
}

// Launch+Snipe configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchSnipeConfig {
    pub enable_launch_snipe: bool,
    pub buy_amount_percentage: f64, // % of liquidity to buy
    pub max_buy_amount_sol: f64,
    pub slippage_bps: u16,
    pub use_jito: bool,
    pub tip_lamports: u64,
    pub retry_attempts: u32,
    pub timeout_seconds: u64,
}

impl Default for LaunchSnipeConfig {
    fn default() -> Self {
        Self {
            enable_launch_snipe: true,
            buy_amount_percentage: 0.1, // 10% of liquidity
            max_buy_amount_sol: 1.0,    // Max 1 SOL buy
            slippage_bps: 100,          // 1% slippage
            use_jito: true,
            tip_lamports: 500_000,      // 0.0005 SOL tip
            retry_attempts: 3,
            timeout_seconds: 30,
        }
    }
}

// Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchSnipeResponse {
    pub launch_signature: String,
    pub snipe_signature: Option<String>,
    pub bundle_hash: String,
    pub mint_address: String,
    pub launched_at: DateTime<Utc>,
    pub sniped_at: Option<DateTime<Utc>>,
    pub buy_amount: u64,
    pub estimated_profit: Option<f64>,
    pub mev_protected: bool,
}

// Enhanced pump.fun program interface with launch+snipe+bundle
pub struct PumpfunInterface {
    client: Client,
    config: PumpfunConfig,
    jito_config: JitoConfig,
    mev_protection: MevProtection,
    launch_snipe_config: LaunchSnipeConfig,
}

#[derive(Debug, Clone)]
pub struct PumpfunConfig {
    pub enabled: bool,
    pub program_id: String,
    pub associated_program_id: String,
    pub metadata_program_id: String,
    pub system_program_id: String,
    pub rent_program_id: String,
    pub token_program_id: String,
    pub associated_token_program_id: String,
    pub pumpfun_program_id: String,
}

impl Default for PumpfunConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            program_id: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(),
            associated_program_id: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string(),
            metadata_program_id: "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".to_string(),
            system_program_id: "11111111111111111111111111111112".to_string(),
            rent_program_id: "11111111111111111111111111111112".to_string(),
            token_program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            associated_token_program_id: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string(),
            pumpfun_program_id: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(),
        }
    }
}

impl PumpfunInterface {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            config: PumpfunConfig::default(),
            jito_config: JitoConfig::default(),
            mev_protection: MevProtection::default(),
            launch_snipe_config: LaunchSnipeConfig::default(),
        }
    }

    /// Enhanced bundle transaction creation with Jito integration
    pub async fn create_bundle_transaction(
        &self,
        from_public_key: String,
        private_key: Vec<u8>,
        recipient: String,
        amount: u64,
        network: String,
        use_jito: bool,
        tip_lamports: Option<u64>,
    ) -> Result<String, PumpfunBundlerError> {
        let keypair = Keypair::from_bytes(&private_key)
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        let to_pubkey = Pubkey::from_str(&recipient)
            .map_err(|e| PumpfunBundlerError::InvalidMetadata(e.to_string()))?;

        let rpc_url = match network.as_str() {
            "mainnet" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            _ => return Err(PumpfunBundlerError::InvalidMetadata("Invalid network".to_string())),
        };

        let rpc_client = RpcClient::new(rpc_url.to_string());
        let recent_blockhash = rpc_client.get_latest_blockhash()
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        let instruction = system_instruction::transfer(&keypair.pubkey(), &to_pubkey, amount);
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );

        if use_jito && self.jito_config.enabled {
            self.send_via_jito_bundle(vec![transaction], tip_lamports).await
        } else {
            let signature = rpc_client.send_and_confirm_transaction(&transaction)
                .map_err(|e| PumpfunBundlerError::BundleSubmission(e.to_string()))?;
            Ok(signature.to_string())
        }
    }

    /// Send multiple transactions as a Jito bundle
    async fn send_via_jito_bundle(
        &self,
        transactions: Vec<Transaction>,
        tip_lamports: Option<u64>,
    ) -> Result<String, PumpfunBundlerError> {
        let tip = tip_lamports.unwrap_or(self.jito_config.min_tip_lamports);

        // Convert transactions to bundle format
        let mut bundle_transactions = Vec::new();
        for tx in &transactions {
            let serialized = bincode::serialize(tx)
                .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;
            
            let signer_keys = tx.message.static_account_keys()
                .iter()
                .map(|key| key.to_string())
                .collect();

            bundle_transactions.push(BundleTransaction {
                transaction: serialized,
                signer_public_keys: signer_keys,
                version: 0,
            });
        }

        let bundle_request = BundleSubmission {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendBundle".to_string(),
            params: bundle_transactions,
            tip,
        };

        let response = self.client
            .post(&format!("{}/sendBundle", self.jito_config.bundler_url))
            .json(&bundle_request)
            .send()
            .await
            .map_err(|e| PumpfunBundlerError::JitoApi(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PumpfunBundlerError::JitoApi(format!("HTTP {}", response.status())));
        }

        let result: serde_json::Value = response.json()
            .await
            .map_err(|e| PumpfunBundlerError::JitoApi(e.to_string()))?;

        if let Some(bundle_hash) = result.get("result").and_then(|r| r.as_str()) {
            Ok(format!("bundle:{}", bundle_hash))
        } else {
            Err(PumpfunBundlerError::JitoApi("Invalid response from Jito".to_string()))
        }
    }

    /// Enhanced pump.fun token creation
    pub async fn create_pump_fun_token(
        &self,
        dev_wallet: Wallet,
        password: String,
        metadata: PumpfunTokenMetadata,
        network: String,
        use_jito: Option<bool>,
    ) -> Result<PumpfunTokenResponse, PumpfunBundlerError> {
        if metadata.name.is_empty() || metadata.symbol.is_empty() {
            return Err(PumpfunBundlerError::InvalidMetadata("Name and symbol are required".to_string()));
        }

        if metadata.initial_liquidity_sol <= 0.0 {
            return Err(PumpfunBundlerError::InvalidMetadata("Initial liquidity must be positive".to_string()));
        }

        let encryption_key = derive_encryption_key(&password, &dev_wallet.salt)?;
        let private_key_bytes = decrypt_data(&encryption_key, &dev_wallet.encrypted_private_key)?;
        let keypair = Keypair::from_bytes(&private_key_bytes)
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        let rpc_url = match network.as_str() {
            "mainnet" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            _ => return Err(PumpfunBundlerError::InvalidMetadata("Invalid network".to_string())),
        };

        let rpc_client = RpcClient::new(rpc_url.to_string());
        let recent_blockhash = rpc_client.get_latest_blockhash()
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        let mint_keypair = Keypair::new();
        let mint_rent = rpc_client.get_minimum_balance_for_rent_exemption(82)
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        let initial_liquidity_lamports = (metadata.initial_liquidity_sol * 1_000_000_000.0) as u64;

        // Create pump.fun launch transaction
        let launch_instructions = self.create_pumpfun_launch_instructions(
            &keypair,
            &mint_keypair,
            &metadata,
            initial_liquidity_lamports,
        )?;

        let launch_transaction = Transaction::new_signed_with_payer(
            &launch_instructions,
            Some(&keypair.pubkey()),
            &[&keypair, &mint_keypair],
            recent_blockhash,
        );

        // Simulate launch transaction
        if self.mev_protection.simulate_before_submit {
            let simulation = rpc_client.simulate_transaction(&launch_transaction)
                .map_err(|e| PumpfunBundlerError::Simulation(e.to_string()))?;
            
            if simulation.value.err.is_some() {
                return Err(PumpfunBundlerError::Simulation(
                    simulation.value.err.unwrap().to_string()
                ));
            }
        }

        // Send launch transaction
        let signature = if use_jito.unwrap_or(false) && self.jito_config.enabled {
            let bundle_result = self.send_via_jito_bundle(vec![launch_transaction], Some(self.jito_config.min_tip_lamports)).await?;
            bundle_result
        } else {
            rpc_client.send_and_confirm_transaction(&launch_transaction)
                .map_err(|e| PumpfunBundlerError::BundleSubmission(e.to_string()))?
                .to_string()
        };

        Ok(PumpfunTokenResponse {
            mint_address: mint_keypair.pubkey().to_string(),
            signature,
            metadata_url: self.upload_metadata(&metadata).await?,
            created_at: Utc::now(),
            initial_liquidity: initial_liquidity_lamports,
        })
    }

    /// Create pump.fun launch instructions
    fn create_pumpfun_launch_instructions(
        &self,
        payer: &Keypair,
        mint_keypair: &Keypair,
        metadata: &PumpfunTokenMetadata,
        initial_liquidity: u64,
    ) -> Result<Vec<solana_sdk::instruction::Instruction>, PumpfunBundlerError> {
        // This would implement the actual pump.fun program instructions
        // For now, creating basic token creation instructions
        
        let mut instructions = Vec::new();

        // Create mint account
        let create_mint_ix = system_instruction::create_account(
            &payer.pubkey(),
            &mint_keypair.pubkey(),
            82,
            &solana_sdk::system_instruction::SystemInstruction::CreateAccount
        );

        // Initialize mint
        let init_mint_ix = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_keypair.pubkey(),
            &payer.pubkey(),
            None,
            9,
        ).map_err(|e| PumpfunBundlerError::PumpfunProgram(e.to_string()))?;

        instructions.extend(vec![create_mint_ix, init_mint_ix]);

        Ok(instructions)
    }

    /// Launch+Snipe+Bundle: The critical MEV protection strategy
    pub async fn launch_snipe_bundle(
        &self,
        dev_wallet: Wallet,
        password: String,
        metadata: PumpfunTokenMetadata,
        network: String,
        config: Option<LaunchSnipeConfig>,
    ) -> Result<LaunchSnipeResponse, PumpfunBundlerError> {
        let launch_config = config.unwrap_or_else(|| self.launch_snipe_config.clone());
        
        if !launch_config.enable_launch_snipe {
            return Err(PumpfunBundlerError::LaunchSnipeFailed("Launch+Snipe disabled".to_string()));
        }

        // Decrypt dev wallet
        let encryption_key = derive_encryption_key(&password, &dev_wallet.salt)?;
        let private_key_bytes = decrypt_data(&encryption_key, &dev_wallet.encrypted_private_key)?;
        let keypair = Keypair::from_bytes(&private_key_bytes)
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        let rpc_url = match network.as_str() {
            "mainnet" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            _ => return Err(PumpfunBundlerError::InvalidMetadata("Invalid network".to_string())),
        };

        let rpc_client = RpcClient::new(rpc_url.to_string());
        let recent_blockhash = rpc_client.get_latest_blockhash()
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        // Generate mint keypair for predictable address
        let mint_keypair = Keypair::new();
        
        // Create launch transaction
        let initial_liquidity_lamports = (metadata.initial_liquidity_sol * 1_000_000_000.0) as u64;
        let launch_instructions = self.create_pumpfun_launch_instructions(
            &keypair,
            &mint_keypair,
            &metadata,
            initial_liquidity_lamports,
        )?;

        let launch_transaction = Transaction::new_signed_with_payer(
            &launch_instructions,
            Some(&keypair.pubkey()),
            &[&keypair, &mint_keypair],
            recent_blockhash,
        );

        // Calculate snipe amount
        let snipe_amount_lamports = ((initial_liquidity_lamports as f64) * launch_config.buy_amount_percentage * 1_000_000_000.0) as u64;
        let final_snipe_amount = snipe_amount_lamports.min((launch_config.max_buy_amount_sol * 1_000_000_000.0) as u64);

        // Create snipe transaction (buy the token immediately)
        let snipe_instructions = self.create_snipe_instructions(
            &keypair,
            &mint_keypair.pubkey(),
            final_snipe_amount,
            launch_config.slippage_bps,
        )?;

        let snipe_transaction = Transaction::new_signed_with_payer(
            &snipe_instructions,
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );

        // Apply MEV protection timing
        if self.mev_protection.randomize_timing {
            let random_delay = OsRng.gen_range(100..300);
            tokio::time::sleep(Duration::from_millis(random_delay)).await;
        }

        // Create bundle with both transactions atomically
        let transactions = if launch_config.use_jito && self.jito_config.enabled {
            // Use Jito bundle for MEV protection
            vec![launch_transaction, snipe_transaction]
        } else {
            // Fallback: send sequentially with minimal delay
            rpc_client.send_transaction(&launch_transaction)
                .map_err(|e| PumpfunBundlerError::LaunchSnipeFailed(e.to_string()))?;
            
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            let snipe_signature = rpc_client.send_transaction(&snipe_transaction)
                .map_err(|e| PumpfunBundlerError::SnipeFailed(e.to_string()))?;
            
            return Ok(LaunchSnipeResponse {
                launch_signature: "pending".to_string(), // Would get from previous call
                snipe_signature: Some(snipe_signature.to_string()),
                bundle_hash: "fallback".to_string(),
                mint_address: mint_keypair.pubkey().to_string(),
                launched_at: Utc::now(),
                sniped_at: Some(Utc::now()),
                buy_amount: final_snipe_amount,
                estimated_profit: None,
                mev_protected: false,
            });
        };

        // Submit bundle via Jito
        let bundle_result = self.send_via_jito_bundle(
            transactions,
            Some(launch_config.tip_lamports)
        ).await?;

        Ok(LaunchSnipeResponse {
            launch_signature: bundle_result.clone(),
            snipe_signature: Some(bundle_result),
            bundle_hash: bundle_result,
            mint_address: mint_keypair.pubkey().to_string(),
            launched_at: Utc::now(),
            sniped_at: Some(Utc::now()),
            buy_amount: final_snipe_amount,
            estimated_profit: self.estimate_profit(&metadata, final_snipe_amount).await,
            mev_protected: true,
        })
    }

    /// Create snipe instructions (buy token immediately after launch)
    fn create_snipe_instructions(
        &self,
        wallet: &Keypair,
        mint_address: &Pubkey,
        amount_lamports: u64,
        slippage_bps: u16,
    ) -> Result<Vec<solana_sdk::instruction::Instruction>, PumpfunBundlerError> {
        // This would implement the actual snipe logic
        // For now, creating basic swap instructions
        
        let mut instructions = Vec::new();
        
        // TODO: Implement actual pump.fun snipe program instructions
        // This would involve:
        // 1. Converting SOL to the new token
        // 2. Using pump.fun's swap program
        // 3. Handling slippage protection
        
        // Placeholder: Just transfer to self as placeholder
        let instruction = system_instruction::transfer(
            &wallet.pubkey(),
            &wallet.pubkey(),
            amount_lamports,
        );
        
        instructions.push(instruction);
        
        Ok(instructions)
    }

    /// Estimate potential profit from launch+snipe
    async fn estimate_profit(&self, metadata: &PumpfunTokenMetadata, buy_amount: u64) -> Option<f64> {
        // This would analyze:
        // 1. Market conditions
        // 2. Token potential
        // 3. Historical pump.fun launches
        // 4. Social metrics
        
        // Placeholder calculation
        let initial_liquidity_sol = metadata.initial_liquidity_sol;
        let buy_amount_sol = buy_amount as f64 / 1_000_000_000.0;
        
        // Simple estimate: 10x potential (highly speculative)
        Some(buy_amount_sol * 10.0 - buy_amount_sol)
    }

    /// Upload metadata to decentralized storage
    async fn upload_metadata(&self, metadata: &PumpfunTokenMetadata) -> Result<String, PumpfunBundlerError> {
        // In reality, this would upload to Arweave, IPFS, or similar
        // For now, return a placeholder
        Ok("https://arweave.net/placeholder".to_string())
    }

    /// Enhanced bundle buying with MEV protection
    pub async fn execute_bundle_buy(
        &self,
        bundle_wallets: Vec<Wallet>,
        token_address: String,
        amount_per_wallet: u64,
        password: String,
        swap_dapp: SwapDapp,
        network: String,
        use_mev_protection: bool,
    ) -> Result<BundleBuyResponse, PumpfunBundlerError> {
        let mut signatures = Vec::new();
        let start_time = Instant::now();

        if bundle_wallets.is_empty() {
            return Err(PumpfunBundlerError::InvalidMetadata("No bundle wallets provided".to_string()));
        }

        // Use Jito bundles for better MEV protection
        let mut transactions = Vec::new();
        for wallet in &bundle_wallets {
            let encryption_key = derive_encryption_key(&password, &wallet.salt)?;
            let private_key_bytes = decrypt_data(&encryption_key, &wallet.encrypted_private_key)?;
            let keypair = Keypair::from_bytes(&private_key_bytes)
                .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

            let transaction = self.create_swap_transaction(
                &keypair,
                &token_address,
                amount_per_wallet,
                &swap_dapp,
                &network,
            ).await?;

            transactions.push(transaction);
        }

        // Send all transactions as a bundle
        if use_mev_protection && self.jito_config.enabled {
            let bundle_result = self.send_via_jito_bundle(
                transactions,
                Some(self.jito_config.min_tip_lamports)
            ).await?;
            
            signatures.push(bundle_result);
        } else {
            for transaction in transactions {
                let rpc_client = RpcClient::new(self.get_rpc_url(&network));
                let signature = rpc_client.send_and_confirm_transaction(&transaction)
                    .map_err(|e| PumpfunBundlerError::BundleSubmission(e.to_string()))?;
                signatures.push(signature.to_string());
                
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(BundleBuyResponse {
            signatures,
            total_transactions: bundle_wallets.len(),
            total_amount: amount_per_wallet * bundle_wallets.len() as u64,
            execution_time: start_time.elapsed(),
            success_count: signatures.len(),
        })
    }

    /// Create swap transaction for bundle buying
    async fn create_swap_transaction(
        &self,
        wallet: &Keypair,
        token_address: &str,
        amount: u64,
        swap_dapp: &SwapDapp,
        network: &str,
    ) -> Result<Transaction, PumpfunBundlerError> {
        // This would integrate with actual swap APIs
        // For now, create a simple transfer as placeholder
        
        let token_pubkey = Pubkey::from_str(token_address)
            .map_err(|e| PumpfunBundlerError::InvalidMetadata(e.to_string()))?;

        let rpc_client = RpcClient::new(self.get_rpc_url(network));
        let recent_blockhash = rpc_client.get_latest_blockhash()
            .map_err(|e| PumpfunBundlerError::BundleConstruction(e.to_string()))?;

        let instruction = system_instruction::transfer(&wallet.pubkey(), &token_pubkey, amount);

        Ok(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&wallet.pubkey()),
            &[wallet],
            recent_blockhash,
        ))
    }

    /// Get RPC URL for network
    fn get_rpc_url(&self, network: &str) -> String {
        match network {
            "mainnet" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            _ => "https://api.mainnet-beta.solana.com".to_string(),
        }
    }
}

// Supporting structures (keeping existing ones)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapDapp {
    Jupiter,
    Photon,
    Orca,
    Raydium,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpfunTokenResponse {
    pub mint_address: String,
    pub signature: String,
    pub metadata_url: String,
    pub created_at: DateTime<Utc>,
    pub initial_liquidity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleBuyResponse {
    pub signatures: Vec<String>,
    pub total_transactions: usize,
    pub total_amount: u64,
    pub execution_time: Duration,
    pub success_count: usize,
}

impl From<PumpfunBundlerError> for String {
    fn from(err: PumpfunBundlerError) -> String {
        err.to_string()
    }
}