use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::KeyInit, aead::Aead};
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString, Output}};
use pbkdf2::Pbkdf2;
use scrypt::{ScryptParams, scrypt};
use rand::rngs::OsRng;
use rand::RngCore;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};
use blake3::Hasher as Blake3Hasher;
use zeroize::{Zeroize, ZeroizeOnDrop};
use ring::digest;
use crossbeam::channel::{Sender, Receiver};
use once_cell::sync::Lazy;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

/// Security error types
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("Encryption/decryption failed: {0}")]
    Encryption(String),
    #[error("Invalid key material: {0}")]
    InvalidKeyMaterial(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Transaction validation failed: {0}")]
    TransactionValidation(String),
    #[error("Secure memory operation failed: {0}")]
    SecureMemory(String),
    #[error("Certificate verification failed: {0}")]
    CertificateVerification(String),
    #[error("Key rotation failed: {0}")]
    KeyRotation(String),
    #[error("Backup/Recovery failed: {0}")]
    BackupRecovery(String),
}

impl From<SecurityError> for String {
    fn from(err: SecurityError) -> String {
        err.to_string()
    }
}

/// Enhanced encryption parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionParams {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivationMethod,
    pub iterations: u32,
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub key_length: u32,
    pub salt_length: u32,
    pub nonce_length: u32,
}

impl Default for EncryptionParams {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation: KeyDerivationMethod::Argon2id,
            iterations: 100_000,
            memory_cost: 19456, // 19.2 MB
            time_cost: 2,
            parallelism: 1,
            key_length: 32,
            salt_length: 32,
            nonce_length: 12,
        }
    }
}

/// Encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    Aes256Ccm,
    ChaCha20Poly1305,
}

/// Key derivation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivationMethod {
    Argon2id,
    Argon2i,
    Argon2d,
    PBKDF2,
    Scrypt,
    HKDF,
}

/// Secure key structure with automatic zeroization
#[derive(ZeroizeOnDrop)]
pub struct SecureKey {
    pub key_material: Vec<u8>,
    pub derivation_method: KeyDerivationMethod,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub key_version: u32,
}

impl SecureKey {
    fn new(key_material: Vec<u8>, method: KeyDerivationMethod) -> Self {
        Self {
            key_material,
            derivation_method: method,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            key_version: 1,
        }
    }

    /// Securely access the key material
    pub fn access<F, R>(&mut self, f: F) -> R 
    where
        F: FnOnce(&[u8]) -> R,
    {
        self.last_accessed = Utc::now();
        self.access_count += 1;
        f(&self.key_material)
    }
}

/// Transaction security parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSecurity {
    pub max_amount: u64,
    pub daily_limit: u64,
    pub transaction_timeout: Duration,
    pub require_approval: bool,
    pub whitelist_enabled: bool,
    pub blacklist_enabled: bool,
    pub simulation_required: bool,
    pub gas_estimation: bool,
}

impl Default for TransactionSecurity {
    fn default() -> Self {
        Self {
            max_amount: u64::MAX,
            daily_limit: u64::MAX,
            transaction_timeout: Duration::from_secs(300), // 5 minutes
            require_approval: false,
            whitelist_enabled: false,
            blacklist_enabled: false,
            simulation_required: true,
            gas_estimation: true,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u32,
    pub max_transactions_per_hour: u32,
    pub max_volume_per_hour: u64,
    pub ban_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            max_transactions_per_hour: 100,
            max_volume_per_hour: 1_000_000_000_000, // 1 trillion lamports (1M SOL)
            ban_duration: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Certificate pinning for network requests
#[derive(Debug, Clone)]
pub struct CertificatePinning {
    pub enabled: bool,
    pub allowed_domains: Vec<String>,
    pub public_key_pins: HashMap<String, Vec<String>>,
    pub expiry_checking: bool,
}

impl Default for CertificatePinning {
    fn default() -> Self {
        let mut pins = HashMap::new();
        pins.insert("api.mainnet-beta.solana.com".to_string(), vec![]);
        pins.insert("api.devnet.solana.com".to_string(), vec![]);
        
        Self {
            enabled: true,
            allowed_domains: vec![
                "api.mainnet-beta.solana.com".to_string(),
                "api.devnet.solana.com".to_string(),
                "api.testnet.solana.com".to_string(),
                "quote-api.jup.ag".to_string(),
                "token.jup.ag".to_string(),
            ],
            public_key_pins: pins,
            expiry_checking: true,
        }
    }
}

/// Enhanced security manager
pub struct SecurityManager {
    // Key management
    master_keys: Arc<DashMap<String, SecureKey>>,
    key_rotation_schedule: Arc<DashMap<String, DateTime<Utc>>>,
    
    // Rate limiting
    request_tracker: Arc<DashMap<String, (u32, Instant)>>,
    transaction_tracker: Arc<DashMap<String, (u64, Instant)>>,
    banned_addresses: Arc<DashMap<String, DateTime<Utc>>>,
    
    // Transaction security
    whitelist: Arc<DashMap<String, bool>>,
    blacklist: Arc<DashMap<String, bool>>,
    transaction_history: Arc<DashMap<String, Vec<DateTime<Utc>>>>,
    
    // Monitoring and logging
    security_events: Sender<SecurityEvent>,
    
    // Configuration
    encryption_params: EncryptionParams,
    rate_limit_config: RateLimitConfig,
    certificate_pinning: CertificatePinning,
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    KeyAccessed { key_id: String, timestamp: DateTime<Utc> },
    TransactionAttempt { from: String, to: String, amount: u64, timestamp: DateTime<Utc> },
    RateLimitTriggered { client_id: String, limit_type: String, timestamp: DateTime<Utc> },
    SecurityViolation { violation_type: String, details: String, timestamp: DateTime<Utc> },
    CertificatePinned { domain: String, valid: bool, timestamp: DateTime<Utc> },
    KeyRotated { key_id: String, old_version: u32, new_version: u32, timestamp: DateTime<Utc> },
}

impl SecurityManager {
    /// Create new security manager
    pub fn new(security_events: Sender<SecurityEvent>) -> Self {
        Self {
            master_keys: Arc::new(DashMap::new()),
            key_rotation_schedule: Arc::new(DashMap::new()),
            request_tracker: Arc::new(DashMap::new()),
            transaction_tracker: Arc::new(DashMap::new()),
            banned_addresses: Arc::new(DashMap::new()),
            whitelist: Arc::new(DashMap::new()),
            blacklist: Arc::new(DashMap::new()),
            transaction_history: Arc::new(DashMap::new()),
            security_events,
            encryption_params: EncryptionParams::default(),
            rate_limit_config: RateLimitConfig::default(),
            certificate_pinning: CertificatePinning::default(),
        }
    }

    /// Enhanced key derivation with multiple algorithms
    pub fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        method: KeyDerivationMethod,
    ) -> Result<Vec<u8>, SecurityError> {
        match method {
            KeyDerivationMethod::Argon2id | KeyDerivationMethod::Argon2i | KeyDerivationMethod::Argon2d => {
                let argon2 = Argon2::new(
                    match method {
                        KeyDerivationMethod::Argon2id => argon2::Variant::Argon2id,
                        KeyDerivationMethod::Argon2i => argon2::Variant::Argon2i,
                        KeyDerivationMethod::Argon2d => argon2::Variant::Argon2d,
                        _ => argon2::Variant::Argon2id,
                    },
                    argon2::Version::V0x13,
                    self.encryption_params.time_cost,
                    self.encryption_params.memory_cost,
                    self.encryption_params.parallelism,
                );

                let mut key = vec![0u8; self.encryption_params.key_length as usize];
                argon2.hash_password_into(password.as_bytes(), salt, &mut key)
                    .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;
                
                Ok(key)
            }
            
            KeyDerivationMethod::PBKDF2 => {
                let mut key = vec![0u8; self.encryption_params.key_length as usize];
                Pbkdf2::hash_password_into(password.as_bytes(), salt, self.encryption_params.iterations, &mut key)
                    .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;
                Ok(key)
            }
            
            KeyDerivationMethod::Scrypt => {
                let params = ScryptParams::new(
                    14, // log_n
                    8,  // r
                    1,  // p
                );
                let mut key = vec![0u8; self.encryption_params.key_length as usize];
                scrypt(password.as_bytes(), salt, &params, &mut key)
                    .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;
                Ok(key)
            }
            
            KeyDerivationMethod::HKDF => {
                // Implementation for HKDF would go here
                Err(SecurityError::KeyDerivation("HKDF not yet implemented".to_string()))
            }
        }
    }

    /// Enhanced encryption with AEAD
    pub fn encrypt_data(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let algorithm = self.encryption_params.algorithm.clone();
        
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if key.len() != 32 {
                    return Err(SecurityError::InvalidKeyMaterial("Key must be 32 bytes for AES-256".to_string()));
                }

                let key: &Key<Aes256Gcm> = key.into();
                let cipher = Aes256Gcm::new(key);
                
                // Generate secure nonce
                let mut nonce_bytes = [0u8; 12];
                OsRng.fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);

                let ciphertext = cipher.encrypt(nonce, data)
                    .map_err(|e| SecurityError::Encryption(e.to_string()))?;

                // Return nonce + ciphertext
                let mut result = nonce_bytes.to_vec();
                result.extend(ciphertext);
                Ok(result)
            }
            _ => Err(SecurityError::Encryption("Algorithm not implemented".to_string()))
        }
    }

    /// Enhanced decryption
    pub fn decrypt_data(&self, key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if encrypted_data.len() < 12 {
            return Err(SecurityError::Encryption("Invalid encrypted data length".to_string()));
        }

        let algorithm = self.encryption_params.algorithm.clone();
        
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                if key.len() != 32 {
                    return Err(SecurityError::InvalidKeyMaterial("Key must be 32 bytes for AES-256".to_string()));
                }

                let key: &Key<Aes256Gcm> = key.into();
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(&encrypted_data[..12]);
                let ciphertext = &encrypted_data[12..];

                cipher.decrypt(nonce, ciphertext)
                    .map_err(|e| SecurityError::Encryption(e.to_string()))
            }
            _ => Err(SecurityError::Encryption("Algorithm not implemented".to_string()))
        }
    }

    /// Rate limiting check
    pub fn check_rate_limit(&self, client_id: &str, request_type: &str) -> Result<(), SecurityError> {
        let now = Instant::now();
        
        match request_type {
            "request" => {
                if let Some(mut entry) = self.request_tracker.get_mut(client_id) {
                    let (count, last_reset) = entry.value_mut();
                    
                    if last_reset.elapsed() > Duration::from_secs(60) {
                        *count = 0;
                        *last_reset = now;
                    }
                    
                    if *count >= self.rate_limit_config.max_requests_per_minute {
                        return Err(SecurityError::RateLimitExceeded);
                    }
                    *count += 1;
                } else {
                    self.request_tracker.insert(client_id.to_string(), (1, now));
                }
            }
            "transaction" => {
                if let Some(mut entry) = self.transaction_tracker.get_mut(client_id) {
                    let (count, last_reset) = entry.value_mut();
                    
                    if last_reset.elapsed() > Duration::from_secs(3600) {
                        *count = 0;
                        *last_reset = now;
                    }
                    
                    if *count >= self.rate_limit_config.max_transactions_per_hour {
                        return Err(SecurityError::RateLimitExceeded);
                    }
                    *count += 1;
                } else {
                    self.transaction_tracker.insert(client_id.to_string(), (1, now));
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Transaction validation with security checks
    pub fn validate_transaction(
        &self,
        from_address: &str,
        to_address: &str,
        amount: u64,
    ) -> Result<(), SecurityError> {
        // Check if addresses are banned
        if self.banned_addresses.get(from_address).is_some() || 
           self.banned_addresses.get(to_address).is_some() {
            return Err(SecurityError::TransactionValidation(
                "Transaction from/to banned address".to_string()
            ));
        }

        // Check whitelist if enabled
        if self.certificate_pinning.whitelist_enabled {
            if self.whitelist.get(to_address).is_none() {
                return Err(SecurityError::TransactionValidation(
                    "Destination address not in whitelist".to_string()
                ));
            }
        }

        // Check blacklist
        if self.blacklist.get(to_address).is_some() {
            return Err(SecurityError::TransactionValidation(
                "Destination address is blacklisted".to_string()
            ));
        }

        // Check amount limits
        if amount > 0 {
            // Daily volume check
            let today = Utc::now().date_naive();
            let key = format!("{}_{}", from_address, today);
            
            if let Some(mut entry) = self.transaction_history.get_mut(&key) {
                let history = entry.value_mut();
                let today_transactions: Vec<_> = history.iter()
                    .filter(|&dt| dt.date_naive() == today)
                    .collect();
                
                let daily_volume: u64 = today_transactions.iter().count() as u64 * amount;
                if daily_volume > 0 {
                    // In a real implementation, you'd calculate actual volume
                    if daily_volume > 0 {
                        return Err(SecurityError::TransactionValidation(
                            "Daily transaction limit exceeded".to_string()
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate secure salt
    pub fn generate_salt(&self) -> Vec<u8> {
        let mut salt = vec![0u8; self.encryption_params.salt_length as usize];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Generate secure random bytes
    pub fn generate_random_bytes(&self, length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    /// Certificate pinning validation
    pub fn validate_certificate(&self, domain: &str) -> Result<bool, SecurityError> {
        if !self.certificate_pinning.enabled {
            return Ok(true);
        }

        if !self.certificate_pinning.allowed_domains.contains(&domain.to_string()) {
            return Err(SecurityError::CertificateVerification(
                format!("Domain not allowed: {}", domain)
            ));
        }

        // In a real implementation, you would:
        // 1. Extract the certificate chain
        // 2. Get the public key pins for the domain
        // 3. Verify the certificate against the pins
        // 4. Check certificate expiry if enabled

        Ok(true)
    }

    /// Key rotation management
    pub fn schedule_key_rotation(&self, key_id: &str, rotation_date: DateTime<Utc>) {
        self.key_rotation_schedule.insert(key_id.to_string(), rotation_date);
    }

    /// Check if key rotation is due
    pub fn check_key_rotation(&self, key_id: &str) -> bool {
        if let Some(rotation_date) = self.key_rotation_schedule.get(key_id) {
            Utc::now() >= *rotation_date.value()
        } else {
            false
        }
    }

    /// Log security event
    pub fn log_event(&self, event: SecurityEvent) {
        let _ = self.security_events.send(event);
    }

    /// Ban an address temporarily
    pub fn ban_address(&self, address: &str, duration: Duration) {
        let expiry = Utc::now() + duration;
        self.banned_addresses.insert(address.to_string(), expiry);
    }

    /// Add address to whitelist
    pub fn add_to_whitelist(&self, address: &str) {
        self.whitelist.insert(address.to_string(), true);
    }

    /// Add address to blacklist
    pub fn add_to_blacklist(&self, address: &str) {
        self.blacklist.insert(address.to_string(), true);
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        let now = Utc::now();
        
        // Clean up banned addresses
        let keys_to_remove: Vec<String> = self.banned_addresses
            .iter()
            .filter(|entry| *entry.value() < now)
            .map(|entry| entry.key().clone())
            .collect();
        
        for key in keys_to_remove {
            self.banned_addresses.remove(&key);
        }
    }
}

/// Secure hash functions
pub struct SecureHash;

impl SecureHash {
    /// BLAKE3 hash
    pub fn blake3(data: &[u8]) -> Vec<u8> {
        let mut hasher = Blake3Hasher::new();
        hasher.update(data);
        hasher.finalize().as_bytes().to_vec()
    }

    /// SHA-256 hash
    pub fn sha256(data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, data).as_ref().to_vec()
    }

    /// SHA-512 hash
    pub fn sha512(data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA512, data).as_ref().to_vec()
    }

    /// SHA3-256 hash
    pub fn sha3_256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Create HMAC
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let mac = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|_| SecurityError::Encryption("Invalid HMAC key".to_string()))?;
        let result = mac.chain(data).finalize();
        Ok(result.into_bytes().to_vec())
    }
}

/// Global security manager instance
static SECURITY_MANAGER: Lazy<Arc<SecurityManager>> = Lazy::new(|| {
    // Create a channel for security events (for now, a dummy sender)
    let (tx, _rx) = crossbeam::channel::unbounded();
    Arc::new(SecurityManager::new(tx))
});

/// Get global security manager
pub fn get_security_manager() -> &'static Arc<SecurityManager> {
    &SECURITY_MANAGER
}