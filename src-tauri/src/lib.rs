use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signer, SeedDerivable}, system_instruction, stake, stake::instruction as stake_instruction, stake::state::{StakeStateV2, Authorized, Lockup}, program_pack::Pack};
use solana_client::rpc_request::TokenAccountsFilter;
use solana_client::rpc_client::RpcClient;
use bip39::{Mnemonic, Language};
use tauri::command;
use tokio::sync::Mutex;
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use argon2::{Argon2, password_hash::SaltString};
use rand::rngs::OsRng;
use rand::RngCore;
use thiserror::Error;
use chrono::{DateTime, Utc};
use reqwest::Client;
extern crate bincode;
use mpl_token_metadata::accounts::Metadata;
use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey as SolanaPubkey;

// Import our enhanced modules
mod security;
mod performance;
mod monitoring;
mod pumpfun_bundler;

use security::{SecurityManager, get_security_manager, SecurityEvent, SecurityError};
use performance::{PerformanceCache, PerformanceConfig, PerformanceMetrics};
use monitoring::{AnalyticsManager, UserEvent, SystemMetrics};
use pumpfun_bundler::{PumpfunInterface, PumpfunTokenMetadata, PumpfunTokenResponse, BundleBuyResponse, SwapDapp, JitoConfig, MevProtection, PumpfunBundlerError};

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Solana client error: {0}")]
    Solana(String),
    #[error("BIP39 error: {0}")]
    Bip39(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Performance error: {0}")]
    Performance(String),
    #[error("Pumpfun/Bundler error: {0}")]
    PumpfunBundler(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub encrypted_private_key: Vec<u8>,
    pub salt: Vec<u8>,
    pub balance: u64,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub network: String,
}

#[derive(Serialize, Deserialize)]
pub struct Wallets {
    pub wallets: Vec<Wallet>,
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionRecord {
    pub signature: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub token_mint: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub network: String,
    pub transaction_type: TransactionType,
    pub fees: u64,
    pub block_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TransactionType {
    Transfer,
    Swap,
    Stake,
    Unstake,
    NFT,
    Bundle,
    Pumpfun,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionHistory {
    pub transactions: Vec<TransactionRecord>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JupiterQuote {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub platform_fee: Option<JupiterPlatformFee>,
    pub price_impact_pct: String,
    pub route_plan: Vec<JupiterRoute>,
    pub context_slot: u64,
    pub time_taken: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JupiterPlatformFee {
    pub amount: String,
    pub fee_bps: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JupiterRoute {
    pub swap_info: JupiterSwapInfo,
    pub percent: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JupiterSwapInfo {
    pub amm_key: String,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

#[derive(Serialize, Deserialize)]
pub struct JupiterSwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: u64,
    pub prioritization_fee_lamports: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StakingAccount {
    pub stake_account: String,
    pub validator: String,
    pub amount: u64,
    pub status: String,
    pub activation_epoch: Option<u64>,
    pub deactivation_epoch: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct StakingAccounts {
    pub accounts: Vec<StakingAccount>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NFT {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub image: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<Vec<NFTAttribute>>,
    pub collection: Option<String>,
    pub update_authority: String,
    pub creators: Vec<NFTCreator>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NFTAttribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NFTCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

#[derive(Serialize, Deserialize)]
pub struct NFTCollection {
    pub nfts: Vec<NFT>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenAccount {
    pub mint: String,
    pub address: String,
    pub amount: u64,
    pub decimals: u8,
    pub ui_amount: f64,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub logo_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenAccounts {
    pub accounts: Vec<TokenAccount>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub mint: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
    pub price: Option<f64>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenList {
    pub tokens: Vec<TokenInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct StakingReward {
    pub epoch: u64,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct StakingRewards {
    pub rewards: Vec<StakingReward>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BundleConfig {
    pub wallet_count: usize,
    pub amount_per_wallet: u64,
    pub use_mev_protection: bool,
    pub use_jito: bool,
    pub tip_lamports: Option<u64>,
    pub swap_dapp: String,
    pub network: String,
}

#[derive(Serialize, Deserialize)]
pub struct BundleExecutionResult {
    pub bundle_id: String,
    pub signatures: Vec<String>,
    pub total_amount: u64,
    pub execution_time: Duration,
    pub success_count: usize,
    pub error_count: usize,
    pub estimated_profit: Option<f64>,
}

#[derive(Clone)]
struct AppState {
    cache: HashMap<String, (u64, Instant)>,
    rate_limiter: HashMap<String, (u32, Instant)>,
    encryption_key: Vec<u8>,
    performance_cache: Arc<PerformanceCache>,
    analytics_manager: Arc<AnalyticsManager>,
    pumpfun_interface: Arc<PumpfunInterface>,
}

impl AppState {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            rate_limiter: HashMap::new(),
            encryption_key: vec![],
            performance_cache: Arc::new(PerformanceCache::new()),
            analytics_manager: Arc::new(AnalyticsManager::new()),
            pumpfun_interface: Arc::new(PumpfunInterface::new()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Devnet,
    Testnet,
}

impl Network {
    fn rpc_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://api.mainnet-beta.solana.com",
            Network::Devnet => "https://api.devnet.solana.com",
            Network::Testnet => "https://api.testnet.solana.com",
        }
    }
}

fn derive_encryption_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, WalletError> {
    let security_manager = get_security_manager();
    match security_manager.derive_key(password, salt, security::KeyDerivationMethod::Argon2id) {
        Ok(key) => Ok(key),
        Err(e) => Err(WalletError::Encryption(e.to_string())),
    }
}

fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

fn encrypt_data(key: &[u8], data: &[u8]) -> Result<Vec<u8>, WalletError> {
    let security_manager = get_security_manager();
    match security_manager.encrypt_data(key, data) {
        Ok(encrypted) => Ok(encrypted),
        Err(e) => Err(WalletError::Encryption(e.to_string())),
    }
}

fn decrypt_data(key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, WalletError> {
    let security_manager = get_security_manager();
    match security_manager.decrypt_data(key, encrypted_data) {
        Ok(decrypted) => Ok(decrypted),
        Err(e) => Err(WalletError::Encryption(e.to_string())),
    }
}

impl From<WalletError> for String {
    fn from(err: WalletError) -> String {
        err.to_string()
    }
}

fn validate_public_key(pubkey_str: &str) -> Result<Pubkey, WalletError> {
    Pubkey::from_str(pubkey_str)
        .map_err(|_| WalletError::InvalidInput("Invalid public key format".to_string()))
}

fn validate_amount(amount: u64) -> Result<(), WalletError> {
    if amount == 0 {
        return Err(WalletError::InvalidInput("Amount must be greater than 0".to_string()));
    }
    Ok(())
}

#[command]
async fn generate_wallet(password: String, network: String) -> Result<Wallet, String> {
    let keypair = Keypair::new();
    let public_key = keypair.pubkey().to_string();
    let private_key = keypair.secret().to_bytes().to_vec();

    // Generate salt for encryption
    let salt = SaltString::generate(&mut OsRng);
    let encryption_key = derive_encryption_key(&password, salt.as_str().as_bytes())?;

    // Encrypt private key
    let encrypted_private_key = encrypt_data(&encryption_key, &private_key)?;

    let now = Utc::now();
    Ok(Wallet {
        public_key,
        encrypted_private_key,
        salt: salt.as_str().as_bytes().to_vec(),
        balance: 0,
        created_at: now,
        last_updated: now,
        network,
    })
}

#[command]
async fn get_balance(
    public_key: String,
    network: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<u64, String> {
    let mut app_state = state.lock().await;
    let security_manager = get_security_manager();

    // Check rate limit
    let client_key = format!("balance_{}_{}", public_key, network);
    if let Err(e) = security_manager.check_rate_limit(&client_key, "request") {
        return Err(e.to_string());
    }

    let (count, last_request) = app_state.rate_limiter.entry(client_key.clone())
        .or_insert((0, Instant::now()));

    if last_request.elapsed() > Duration::from_secs(60) {
        *count = 0;
        *last_request = Instant::now();
    }

    if *count >= 30 {
        return Err(WalletError::RateLimit.to_string());
    }
    *count += 1;

    // Check performance cache
    if let Some(cached_balance) = app_state.performance_cache.get(&format!("balance_{}", client_key)).await {
        return Ok(cached_balance);
    }

    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        "testnet" => Network::Testnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let pubkey = validate_public_key(&public_key)?;

    let balance = rpc_client.get_balance(&pubkey)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    // Cache the result
    app_state.performance_cache.set(
        &format!("balance_{}", client_key), 
        balance, 
        Duration::from_secs(30)
    ).await;

    Ok(balance)
}

#[command]
async fn save_wallets(wallets: Wallets, password: String) -> Result<(), String> {
    // Encrypt the entire wallets structure
    let salt = generate_salt();
    let encryption_key = derive_encryption_key(&password, &salt)?;

    let wallets_json = serde_json::to_string(&wallets)
        .map_err(|e| WalletError::Serde(e))?;

    let encrypted_data = encrypt_data(&encryption_key, wallets_json.as_bytes())?;

    // Store salt and encrypted data
    let mut final_data = salt.to_vec();
    final_data.extend(encrypted_data);

    fs::write("wallets.enc", final_data)
        .map_err(|e| WalletError::Io(e))?;

    Ok(())
}

#[command]
async fn load_wallets(password: String) -> Result<Wallets, String> {
    if !Path::new("wallets.enc").exists() {
        return Ok(Wallets {
            wallets: vec![],
            version: "1.0".to_string(),
        });
    }

    let encrypted_data = fs::read("wallets.enc")
        .map_err(|e| WalletError::Io(e).to_string())?;

    if encrypted_data.len() < 16 {
        return Err(WalletError::Encryption("Invalid encrypted file".to_string()).to_string());
    }

    let salt_bytes = &encrypted_data[..16];
    let encrypted_wallets = &encrypted_data[16..];

    let encryption_key = derive_encryption_key(&password, &salt_bytes)?;

    let decrypted_json = decrypt_data(&encryption_key, encrypted_wallets)?;
    let wallets_json = String::from_utf8(decrypted_json)
        .map_err(|e| WalletError::Encryption(e.to_string()))?;

    let mut wallets: Wallets = serde_json::from_str(&wallets_json)
        .map_err(|e| WalletError::Serde(e))?;

    // Ensure version is set
    if wallets.version.is_empty() {
        wallets.version = "1.0".to_string();
    }

    // Migration: Add salt to wallets that don't have it (backward compatibility)
    for wallet in &mut wallets.wallets {
        if wallet.salt.is_empty() {
            wallet.salt = generate_salt().to_vec();
        }
    }

    Ok(wallets)
}

#[command]
async fn transfer_tokens(
    from_public_key: String,
    encrypted_private_key: Vec<u8>,
    password: String,
    to_public_key: String,
    amount: u64,
    token_mint: Option<String>,
    network: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<String, String> {
    validate_amount(amount)?;

    let mut app_state = state.lock().await;
    let security_manager = get_security_manager();

    // Security validation
    if let Err(e) = security_manager.validate_transaction(&from_public_key, &to_public_key, amount) {
        return Err(e.to_string());
    }

    // Check rate limit for transactions
    let client_key = format!("transfer_{}", from_public_key);
    if let Err(e) = security_manager.check_rate_limit(&client_key, "transaction") {
        return Err(e.to_string());
    }

    let (count, last_request) = app_state.rate_limiter.entry(client_key.clone())
        .or_insert((0, Instant::now()));

    if last_request.elapsed() > Duration::from_secs(60) {
        *count = 0;
        *last_request = Instant::now();
    }

    if *count >= 5 {
        return Err(WalletError::RateLimit.to_string());
    }
    *count += 1;

    // Decrypt private key using stored salt
    let encryption_key = derive_encryption_key(&password, &encrypted_private_key)?;
    let private_key = decrypt_data(&encryption_key, &encrypted_private_key)?;

    let from_keypair = Keypair::from_bytes(&private_key)
        .map_err(|e| WalletError::Encryption(e.to_string()).to_string())?;

    let to_pubkey = validate_public_key(&to_public_key)?;

    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        "testnet" => Network::Testnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let recent_blockhash = rpc_client.get_latest_blockhash()
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let instruction = if let Some(mint) = token_mint {
        let mint_pubkey = validate_public_key(&mint)?;
        let from_ata = spl_associated_token_account::get_associated_token_address(&from_keypair.pubkey(), &mint_pubkey);
        let to_ata = spl_associated_token_account::get_associated_token_address(&to_pubkey, &mint_pubkey);

        spl_token::instruction::transfer(
            &spl_token::id(),
            &from_ata,
            &to_ata,
            &from_keypair.pubkey(),
            &[&from_keypair.pubkey()],
            amount,
        ).map_err(|e| WalletError::Solana(e.to_string()).to_string())?
    } else {
        system_instruction::transfer(&from_keypair.pubkey(), &to_pubkey, amount)
    };

    let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[instruction],
        Some(&from_keypair.pubkey()),
        &[&from_keypair],
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&transaction)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    // Clear balance cache for involved addresses
    app_state.performance_cache.invalidate(&format!("balance_{}_{}", from_public_key, network)).await;
    app_state.performance_cache.invalidate(&format!("balance_{}_{}", to_public_key, network)).await;

    Ok(signature.to_string())
}

#[command]
async fn get_jupiter_quote(
    input_mint: String,
    output_mint: String,
    amount: String,
    slippage_bps: u16
) -> Result<JupiterQuote, String> {
    let client = Client::new();
    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        input_mint, output_mint, amount, slippage_bps
    );

    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to get Jupiter quote: {}", e))?;

    let quote: JupiterQuote = response.json()
        .await
        .map_err(|e| format!("Failed to parse Jupiter quote response: {}", e))?;

    Ok(quote)
}

#[command]
async fn execute_jupiter_swap(
    quote_response: String,
    user_public_key: String,
    wallet: Wallet,
    password: String,
    network: String
) -> Result<String, String> {
    // Decode the quote response
    let quote_data: serde_json::Value = serde_json::from_str(&quote_response)
        .map_err(|e| format!("Invalid quote response: {}", e))?;

    // Get swap transaction from Jupiter
    let client = Client::new();
    let swap_url = "https://quote-api.jup.ag/v6/swap";

    let swap_payload = serde_json::json!({
        "quoteResponse": quote_data,
        "userPublicKey": user_public_key,
        "wrapAndUnwrapSol": true,
        "dynamicComputeUnitLimit": true,
        "prioritizationFeeLamports": "auto"
    });

    let response = client.post(swap_url)
        .header("Content-Type", "application/json")
        .json(&swap_payload)
        .send()
        .await
        .map_err(|e| format!("Failed to get swap transaction: {}", e))?;

    let swap_response: JupiterSwapResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse swap response: {}", e))?;

    Ok(swap_response.swap_transaction)
}

#[command]
async fn send_bundle_transaction(
    public_key: String,
    private_key: Vec<u8>,
    recipient: String,
    amount: u64,
    network: String,
    use_jito: Option<bool>,
    tip_lamports: Option<u64>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<String, String> {
    validate_amount(amount)?;

    let app_state = state.lock().await;
    let pumpfun_interface = &app_state.pumpfun_interface;

    // Use the enhanced pumpfun interface for bundle transactions
    match pumpfun_interface.create_bundle_transaction(
        public_key,
        private_key,
        recipient,
        amount,
        network,
        use_jito.unwrap_or(false),
        tip_lamports
    ).await {
        Ok(signature) => Ok(signature),
        Err(e) => Err(e.to_string())
    }
}

#[command]
async fn create_pump_fun_token(
    dev_wallet: Wallet,
    password: String,
    metadata: PumpfunTokenMetadata,
    network: String,
    use_jito: Option<bool>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<PumpfunTokenResponse, String> {
    let app_state = state.lock().await;
    let pumpfun_interface = &app_state.pumpfun_interface;

    match pumpfun_interface.create_pump_fun_token(
        dev_wallet,
        password,
        metadata,
        network,
        use_jito.unwrap_or(false)
    ).await {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string())
    }
}

#[command]
async fn execute_bundle_buy(
    bundle_wallets: Vec<Wallet>,
    token_address: String,
    amount_per_wallet: u64,
    password: String,
    swap_dapp: String,
    network: String,
    use_mev_protection: Option<bool>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<BundleExecutionResult, String> {
    let app_state = state.lock().await;
    let pumpfun_interface = &app_state.pumpfun_interface;

    // Convert swap dapp string to enum
    let swap_dapp_enum = match swap_dapp.to_lowercase().as_str() {
        "jupiter" => SwapDapp::Jupiter,
        "photon" => SwapDapp::Photon,
        "orca" => SwapDapp::Orca,
        "raydium" => SwapDapp::Raydium,
        _ => SwapDapp::Custom(swap_dapp),
    };

    match pumpfun_interface.execute_bundle_buy(
        bundle_wallets,
        token_address,
        amount_per_wallet,
        password,
        swap_dapp_enum,
        network,
        use_mev_protection.unwrap_or(true)
    ).await {
        Ok(response) => Ok(BundleExecutionResult {
            bundle_id: format!("bundle_{}", Utc::now().timestamp()),
            signatures: response.signatures,
            total_amount: response.total_amount,
            execution_time: response.execution_time,
            success_count: response.success_count,
            error_count: response.total_transactions - response.success_count,
            estimated_profit: None, // Could be calculated based on market data
        }),
        Err(e) => Err(e.to_string())
    }
}

#[command]
async fn get_transaction_history(
    public_key: String,
    network: String,
    limit: Option<usize>
) -> Result<TransactionHistory, String> {
    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let pubkey = validate_public_key(&public_key)?;

    let signatures = rpc_client.get_signatures_for_address(&pubkey)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let limit = limit.unwrap_or(50).min(100);
    let transactions: Vec<TransactionRecord> = signatures.into_iter()
        .take(limit)
        .filter_map(|sig_info| {
            Some(TransactionRecord {
                signature: sig_info.signature,
                from_address: public_key.clone(),
                to_address: "Unknown".to_string(),
                amount: 0,
                token_mint: None,
                timestamp: DateTime::from_timestamp(sig_info.block_time?, 0)?.into(),
                status: if sig_info.confirmation_status.is_some() { "confirmed" } else { "pending" }.to_string(),
                network: network.clone(),
                transaction_type: TransactionType::Transfer,
                fees: 0,
                block_height: sig_info.slot,
            })
        })
        .collect();

    Ok(TransactionHistory { transactions })
}

#[command]
async fn generate_seed_phrase() -> Result<String, String> {
    let mut entropy = [0u8; 32];
    OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy)
        .map_err(|e| WalletError::Bip39(e.to_string()).to_string())?;
    Ok(mnemonic.to_string())
}

#[command]
async fn import_wallet_from_seed_phrase(
    seed_phrase: String,
    password: String,
    network: String
) -> Result<Wallet, String> {
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, &seed_phrase)
        .map_err(|e| WalletError::Bip39(e.to_string()).to_string())?;

    let seed = mnemonic.to_seed("");
    let keypair = Keypair::from_seed(&seed[..32])
        .map_err(|e| WalletError::Bip39(e.to_string()).to_string())?;

    let public_key = keypair.pubkey().to_string();
    let private_key = keypair.secret().to_bytes().to_vec();

    // Encrypt private key
    let salt = SaltString::generate(&mut OsRng);
    let encryption_key = derive_encryption_key(&password, salt.as_str().as_bytes())?;
    let encrypted_private_key = encrypt_data(&encryption_key, &private_key)?;

    let now = Utc::now();
    Ok(Wallet {
        public_key,
        encrypted_private_key,
        salt: salt.as_str().as_bytes().to_vec(),
        balance: 0,
        created_at: now,
        last_updated: now,
        network,
    })
}

#[command]
async fn validate_seed_phrase(seed_phrase: String) -> Result<bool, String> {
    match Mnemonic::parse_in_normalized(Language::English, &seed_phrase) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[command]
async fn get_network_status(network: String) -> Result<serde_json::Value, String> {
    let client = Client::new();
    let url = match network.as_str() {
        "mainnet" => "https://api.mainnet-beta.solana.com",
        "devnet" => "https://api.devnet.solana.com",
        "testnet" => "https://api.testnet.solana.com",
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let response = client.get(url)
        .header("Content-Type", "application/json")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"getVersion"}"#)
        .send()
        .await
        .map_err(|e| WalletError::Network(e.to_string()).to_string())?;

    let status: serde_json::Value = response.json()
        .await
        .map_err(|e| WalletError::Network(e.to_string()).to_string())?;

    Ok(status)
}

#[command]
async fn export_wallet_private_key(
    public_key: String,
    wallet: Wallet,
    password: String
) -> Result<String, String> {
    // Decrypt the private key using stored salt
    let encryption_key = derive_encryption_key(&password, &wallet.salt)?;
    let private_key = decrypt_data(&encryption_key, &wallet.encrypted_private_key)?;

    // Convert to hex string
    let hex_string = hex::encode(&private_key);
    Ok(hex_string)
}

#[command]
async fn import_wallet_from_private_key(
    private_key_hex: String,
    password: String,
    network: String
) -> Result<Wallet, String> {
    // Decode hex private key
    let private_key = hex::decode(&private_key_hex)
        .map_err(|_| WalletError::InvalidInput("Invalid hex private key".to_string()))?;

    // Create keypair from private key bytes
    let keypair = Keypair::from_bytes(&private_key)
        .map_err(|e| WalletError::Bip39(e.to_string()))?;

    let public_key = keypair.pubkey().to_string();

    // Encrypt private key
    let salt = SaltString::generate(&mut OsRng);
    let encryption_key = derive_encryption_key(&password, salt.as_str().as_bytes())?;
    let encrypted_private_key = encrypt_data(&encryption_key, &private_key)?;

    let now = Utc::now();
    Ok(Wallet {
        public_key,
        encrypted_private_key,
        salt: salt.as_str().as_bytes().to_vec(),
        balance: 0,
        created_at: now,
        last_updated: now,
        network,
    })
}

#[command]
async fn get_staking_accounts(
    public_key: String,
    network: String
) -> Result<StakingAccounts, String> {
    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let wallet_pubkey = validate_public_key(&public_key)?;

    let stake_accounts = rpc_client.get_program_accounts(&stake::program::id())
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let mut staking_accounts = Vec::new();

    for (account_pubkey, account) in stake_accounts {
        if let Ok(stake_state) = <StakeStateV2 as BorshDeserialize>::deserialize(&mut &account.data[..]) {
            match stake_state {
                StakeStateV2::Stake(meta, stake, _stake_flags) => {
                    if meta.authorized.staker == wallet_pubkey || meta.authorized.withdrawer == wallet_pubkey {
                        let status = if stake.delegation.activation_epoch != 0 {
                            if stake.delegation.deactivation_epoch == 0 {
                                "active"
                            } else {
                                "deactivating"
                            }
                        } else {
                            "inactive"
                        };

                        staking_accounts.push(StakingAccount {
                            stake_account: account_pubkey.to_string(),
                            validator: stake.delegation.voter_pubkey.to_string(),
                            amount: stake.delegation.stake,
                            status: status.to_string(),
                            activation_epoch: if stake.delegation.activation_epoch != 0 { Some(stake.delegation.activation_epoch) } else { None },
                            deactivation_epoch: if stake.delegation.deactivation_epoch != 0 { Some(stake.delegation.deactivation_epoch) } else { None },
                        });
                    }
                }
                _ => continue,
            }
        }
    }

    Ok(StakingAccounts {
        accounts: staking_accounts,
    })
}

#[command]
async fn delegate_stake(
    wallet: Wallet,
    password: String,
    validator: String,
    amount: u64,
    network: String
) -> Result<String, String> {
    validate_amount(amount)?;
#[command]
async fn launch_snipe_bundle(
    dev_wallet: Wallet,
    password: String,
    metadata: PumpfunTokenMetadata,
    network: String,
    config: Option<LaunchSnipeConfig>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<LaunchSnipeResponse, String> {
    let app_state = state.lock().await;
    let pumpfun_interface = &app_state.pumpfun_interface;

    match pumpfun_interface.launch_snipe_bundle(
        dev_wallet,
        password,
        metadata,
        network,
        config
    ).await {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string())
    }
}

    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());

    // Decrypt private key
    let encryption_key = derive_encryption_key(&password, &wallet.salt)?;
    let private_key_bytes = decrypt_data(&encryption_key, &wallet.encrypted_private_key)?;
    let keypair = Keypair::from_bytes(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let validator_pubkey = validate_public_key(&validator)?;
    let recent_blockhash = rpc_client.get_latest_blockhash()
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    // Create stake account
    let stake_account = Keypair::new();
    let rent = rpc_client.get_minimum_balance_for_rent_exemption(std::mem::size_of::<StakeStateV2>())
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let total_amount = amount + rent;

    let transfer_ix = system_instruction::transfer(&keypair.pubkey(), &stake_account.pubkey(), total_amount);
    let create_stake_ixs = stake_instruction::create_account(
        &keypair.pubkey(),
        &stake_account.pubkey(),
        &Authorized::auto(&keypair.pubkey()),
        &Lockup::default(),
        amount,
    );
    let delegate_ix = stake_instruction::delegate_stake(&stake_account.pubkey(), &keypair.pubkey(), &validator_pubkey);

    let mut instructions = vec![transfer_ix, delegate_ix];
    instructions.extend(create_stake_ixs);

    let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair, &stake_account],
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&transaction)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    Ok(signature.to_string())
}

#[command]
async fn deactivate_stake(
    wallet: Wallet,
    password: String,
    stake_account_address: String,
    network: String
) -> Result<String, String> {
    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());

    // Decrypt private key
    let encryption_key = derive_encryption_key(&password, &wallet.salt)?;
    let private_key_bytes = decrypt_data(&encryption_key, &wallet.encrypted_private_key)?;
    let keypair = Keypair::from_bytes(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let stake_account_pubkey = validate_public_key(&stake_account_address)?;
    let recent_blockhash = rpc_client.get_latest_blockhash()
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let instruction = stake_instruction::deactivate_stake(&stake_account_pubkey, &keypair.pubkey());

    let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[instruction],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&transaction)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    Ok(signature.to_string())
}

#[command]
async fn get_staking_rewards(
    public_key: String,
    network: String,
    epochs: Option<usize>
) -> Result<StakingRewards, String> {
    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        "testnet" => Network::Testnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let wallet_pubkey = validate_public_key(&public_key)?;

    let epochs = epochs.unwrap_or(30);
    let current_epoch = rpc_client.get_epoch_info()
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?
        .epoch;

    let mut rewards = Vec::new();

    for epoch in (current_epoch.saturating_sub(epochs as u64))..=current_epoch {
        if let Ok(epoch_rewards) = rpc_client.get_inflation_reward(&[wallet_pubkey], Some(epoch)) {
            if let Some(reward) = epoch_rewards.into_iter().next().flatten() {
                rewards.push(StakingReward {
                    epoch,
                    amount: reward.amount,
                    timestamp: DateTime::from_timestamp(reward.effective_slot as i64, 0)
                        .unwrap_or_else(|| Utc::now()),
                });
            }
        }
    }

    Ok(StakingRewards { rewards })
}

#[command]
async fn get_nfts_for_wallet(
    public_key: String,
    network: String
) -> Result<NFTCollection, String> {
    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        "testnet" => Network::Testnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let wallet_pubkey = validate_public_key(&public_key)?;

    let mut nfts = Vec::new();

    let token_accounts_filter = TokenAccountsFilter::ProgramId(spl_token::id());
    let token_accounts = rpc_client.get_token_accounts_by_owner(&wallet_pubkey, token_accounts_filter)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    for token_account in token_accounts {
        let account_data = match &token_account.account.data {
            solana_account_decoder::UiAccountData::LegacyBinary(data) => data.as_bytes().to_vec(),
            _ => continue,
        };
        if account_data.len() < 165 { continue; }

        let mut mint_bytes = [0u8; 32];
        mint_bytes.copy_from_slice(&account_data[0..32]);
        let mint_pubkey = Pubkey::new_from_array(mint_bytes);

        if let Ok(mint_account) = rpc_client.get_account(&mint_pubkey) {
            if mint_account.data.len() < 82 { continue; }

            let supply = u64::from_le_bytes(mint_account.data[36..44].try_into().unwrap());
            let decimals = mint_account.data[44];

            if supply == 1 && decimals == 0 {
                let (metadata_pda, _) = Pubkey::find_program_address(
                    &[b"metadata", mpl_token_metadata::ID.as_ref(), mint_pubkey.as_ref()],
                    &mpl_token_metadata::ID,
                );

                if let Ok(metadata_account) = rpc_client.get_account(&metadata_pda) {
                    if let Ok(metadata) = Metadata::deserialize(&mut metadata_account.data.as_slice()) {
                        let client = Client::new();
                        if let Ok(response) = client.get(&metadata.uri).send().await {
                            if let Ok(metadata_json) = response.json::<serde_json::Value>().await {
                                let nft = NFT {
                                    mint: mint_pubkey.to_string(),
                                    name: metadata.name,
                                    symbol: metadata.symbol,
                                    uri: metadata.uri,
                                    image: metadata_json.get("image").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                    description: metadata_json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                    attributes: metadata_json.get("attributes")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| arr.iter()
                                            .filter_map(|attr| {
                                                Some(NFTAttribute {
                                                    trait_type: attr.get("trait_type")?.as_str()?.to_string(),
                                                    value: attr.get("value")?.as_str()?.to_string(),
                                                })
                                            })
                                            .collect()
                                        ),
                                    collection: metadata_json.get("collection")
                                        .and_then(|v| v.get("name"))
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                    update_authority: metadata.update_authority.to_string(),
                                    creators: vec![],
                                };
                                nfts.push(nft);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(NFTCollection { nfts })
}

#[command]
async fn get_nft_metadata(mint_address: String, network: String) -> Result<NFT, String> {
    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        "testnet" => Network::Testnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let mint_pubkey = validate_public_key(&mint_address)?;

    let (metadata_pda, _) = SolanaPubkey::find_program_address(
        &[b"metadata", mpl_token_metadata::ID.as_ref(), mint_pubkey.as_ref()],
        &mpl_token_metadata::ID,
    );

    let metadata_account = rpc_client.get_account(&metadata_pda)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let metadata = Metadata::deserialize(&mut metadata_account.data.as_slice())
        .map_err(|e| format!("Failed to deserialize metadata: {}", e))?;

    let client = Client::new();
    let response = client.get(&metadata.uri).send().await
        .map_err(|e| format!("Failed to fetch metadata: {}", e))?;

    let metadata_json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse metadata JSON: {}", e))?;

    let nft = NFT {
        mint: mint_address,
        name: metadata.name,
        symbol: metadata.symbol,
        uri: metadata.uri,
        image: metadata_json.get("image").and_then(|v| v.as_str()).map(|s| s.to_string()),
        description: metadata_json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        attributes: metadata_json.get("attributes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|attr| {
                    let trait_type = attr.get("trait_type")?.as_str()?;
                    let value = attr.get("value")?.as_str()?;
                    Some(NFTAttribute {
                        trait_type: trait_type.to_string(),
                        value: value.to_string(),
                    })
                })
                .collect()
            ),
        collection: metadata_json.get("collection")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        update_authority: metadata.update_authority.to_string(),
        creators: vec![],
    };

    Ok(nft)
}

#[command]
async fn get_token_accounts(
    public_key: String,
    network: String
) -> Result<TokenAccounts, String> {
    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        "testnet" => Network::Testnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());
    let wallet_pubkey = validate_public_key(&public_key)?;

    let token_accounts_filter = TokenAccountsFilter::ProgramId(spl_token::id());
    let token_accounts = rpc_client.get_token_accounts_by_owner(&wallet_pubkey, token_accounts_filter)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let mut accounts = Vec::new();

    for token_account in token_accounts {
        let account_data = match &token_account.account.data {
            solana_account_decoder::UiAccountData::LegacyBinary(data) => data.as_bytes().to_vec(),
            _ => continue,
        };
        if account_data.len() < 165 { continue; }

        let mut mint_bytes = [0u8; 32];
        mint_bytes.copy_from_slice(&account_data[0..32]);
        let mint = Pubkey::new_from_array(mint_bytes);
        let mut amount_bytes = [0u8; 8];
        amount_bytes.copy_from_slice(&account_data[64..72]);
        let amount = u64::from_le_bytes(amount_bytes);

        if let Ok(mint_account) = rpc_client.get_account(&mint) {
            if mint_account.data.len() >= 45 {
                let decimals = mint_account.data[44];

                accounts.push(TokenAccount {
                    mint: mint.to_string(),
                    address: token_account.pubkey.to_string(),
                    amount,
                    decimals,
                    ui_amount: (amount as f64) / (10u64.pow(decimals as u32) as f64),
                    symbol: None,
                    name: None,
                    logo_uri: None,
                });
            }
        }
    }

    Ok(TokenAccounts { accounts })
}

#[command]
async fn get_token_list(network: String) -> Result<TokenList, String> {
    let client = Client::new();
    let url = "https://token.jup.ag/strict";

    let response = client.get(url).send().await
        .map_err(|e| format!("Failed to fetch token list: {}", e))?;

    let tokens: Vec<serde_json::Value> = response.json().await
        .map_err(|e| format!("Failed to parse token list: {}", e))?;

    let token_list: Vec<TokenInfo> = tokens.into_iter()
        .filter_map(|token| {
            Some(TokenInfo {
                symbol: token.get("symbol")?.as_str()?.to_string(),
                name: token.get("name")?.as_str()?.to_string(),
                mint: token.get("address")?.as_str()?.to_string(),
                decimals: token.get("decimals")?.as_u64()? as u8,
                logo_uri: token.get("logoURI").and_then(|v| v.as_str()).map(|s| s.to_string()),
                price: None,
                tags: token.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|tag| tag.as_str().map(|s| s.to_string()))
                        .collect()
                    )
                    .unwrap_or_default(),
            })
        })
        .collect();

    Ok(TokenList { tokens: token_list })
}

#[command]
async fn sign_message(
    wallet: Wallet,
    password: String,
    message: String,
    network: String
) -> Result<String, String> {
    // Decrypt private key
    let encryption_key = derive_encryption_key(&password, &wallet.salt)?;
    let private_key_bytes = decrypt_data(&encryption_key, &wallet.encrypted_private_key)?;
    let keypair = Keypair::from_bytes(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    // Sign the message
    let signature = keypair.sign_message(message.as_bytes());
    Ok(signature.to_string())
}

#[command]
async fn verify_signature(
    public_key: String,
    message: String,
    signature: String
) -> Result<bool, String> {
    let pubkey = validate_public_key(&public_key)?;
    let sig_bytes = hex::decode(&signature)
        .map_err(|_| "Invalid signature format".to_string())?;
    let sig = solana_sdk::signature::Signature::try_from(sig_bytes.as_slice())
        .map_err(|_| "Invalid signature format".to_string())?;

    let message_bytes = message.as_bytes();
    let verified = sig.verify(&pubkey.to_bytes(), message_bytes);

    Ok(verified)
}

#[command]
async fn simulate_transaction(
    wallet: Wallet,
    password: String,
    recipient: String,
    amount: u64,
    network: String
) -> Result<serde_json::Value, String> {
    validate_amount(amount)?;

    let network_enum = match network.as_str() {
        "mainnet" => Network::Mainnet,
        "devnet" => Network::Devnet,
        "testnet" => Network::Testnet,
        _ => return Err(WalletError::InvalidInput("Invalid network".to_string()).to_string()),
    };

    let rpc_client = RpcClient::new(network_enum.rpc_url().to_string());

    // Decrypt private key
    let encryption_key = derive_encryption_key(&password, &wallet.salt)?;
    let private_key_bytes = decrypt_data(&encryption_key, &wallet.encrypted_private_key)?;
    let keypair = Keypair::from_bytes(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let to_pubkey = validate_public_key(&recipient)?;
    let recent_blockhash = rpc_client.get_latest_blockhash()
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let instruction = system_instruction::transfer(&keypair.pubkey(), &to_pubkey, amount);
    let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[instruction],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    let simulation = rpc_client.simulate_transaction(&transaction)
        .map_err(|e| WalletError::Solana(e.to_string()).to_string())?;

    let result = serde_json::json!({
        "success": simulation.value.err.is_none(),
        "error": simulation.value.err,
        "logs": simulation.value.logs,
        "accounts": simulation.value.accounts,
        "units_consumed": simulation.value.units_consumed,
        "return_data": simulation.value.return_data
    });

    Ok(result)
}

#[command]
async fn check_for_updates() -> Result<serde_json::Value, String> {
    let client = Client::new();

    let url = "https://api.github.com/repos/lesinski-tools/lesinki-wallet/releases/latest";

    let response = client
        .get(url)
        .header("User-Agent", "Lesinki-Wallet/1.0.0")
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    if !response.status().is_success() {
        return Ok(serde_json::json!({
            "update_available": false,
            "error": "Failed to fetch release information"
        }));
    }

    let release_info: serde_json::Value = response.json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let latest_version = release_info.get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let current_version = env!("CARGO_PKG_VERSION");

    let update_available = latest_version != current_version && latest_version != "unknown";

    Ok(serde_json::json!({
        "update_available": update_available,
        "current_version": current_version,
        "latest_version": latest_version,
        "release_url": release_info.get("html_url").and_then(|v| v.as_str()),
        "release_notes": release_info.get("body").and_then(|v| v.as_str())
    }))
}

#[command]
async fn download_update(download_url: String) -> Result<(), String> {
    let client = Client::new();

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;

    if !response.status().is_success() {
        return Err("Failed to download update file".to_string());
    }

    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read update data: {}", e))?;

    let temp_path = std::env::temp_dir().join("lesinki-wallet-update.msi");
    std::fs::write(&temp_path, bytes)
        .map_err(|e| format!("Failed to save update file: {}", e))?;

    Ok(())
}

#[command]
async fn get_token_price(token_address: String) -> Result<f64, String> {
    let client = Client::new();
    let url = format!("https://api.jup.ag/price/v2?ids={}", token_address);

    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to get price: {}", e))?;

    let price_data: serde_json::Value = response.json()
        .await
        .map_err(|e| format!("Failed to parse price data: {}", e))?;

    let price = price_data
        .get("data")
        .and_then(|d| d.get(&token_address))
        .and_then(|t| t.get("price"))
        .and_then(|p| p.as_str())
        .and_then(|p| p.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(price)
}

#[command]
async fn get_performance_metrics() -> Result<PerformanceMetrics, String> {
    // This would integrate with the performance monitoring system
    Ok(PerformanceMetrics {
        cpu_usage: 0.0,
        memory_usage_mb: 0.0,
        network_latency_ms: 0.0,
        cache_hit_rate: 0.0,
        active_connections: 0,
        requests_per_second: 0.0,
        response_time_ms: 0.0,
        timestamp: Utc::now().timestamp(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            generate_wallet,
            get_balance,
            save_wallets,
            load_wallets,
            transfer_tokens,
            get_transaction_history,
            generate_seed_phrase,
            import_wallet_from_seed_phrase,
            validate_seed_phrase,
            get_network_status,
            export_wallet_private_key,
            import_wallet_from_private_key,
            send_bundle_transaction,
            get_jupiter_quote,
            execute_jupiter_swap,
            get_staking_accounts,
            delegate_stake,
            deactivate_stake,
            get_staking_rewards,
            get_nfts_for_wallet,
            get_nft_metadata,
            sign_message,
            verify_signature,
            simulate_transaction,
            get_token_accounts,
            get_token_list,
            check_for_updates,
            download_update,
            get_token_price,
            get_performance_metrics,
            create_pump_fun_token,
            execute_bundle_buy
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
