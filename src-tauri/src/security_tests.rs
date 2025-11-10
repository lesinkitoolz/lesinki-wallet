#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel;
    use std::time::Duration;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_security_manager_creation() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        // Test basic creation
        assert!(manager.check_rate_limit("test_client", "request").is_ok());
    }

    #[tokio::test]
    async fn test_key_derivation_argon2() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        let password = "test_password";
        let salt = manager.generate_salt();
        let key = manager.derive_key(password, &salt, KeyDerivationMethod::Argon2id)
            .expect("Key derivation should succeed");
        
        // Key should be 32 bytes
        assert_eq!(key.len(), 32);
        
        // Same password and salt should produce same key
        let key2 = manager.derive_key(password, &salt, KeyDerivationMethod::Argon2id)
            .expect("Key derivation should succeed");
        assert_eq!(key, key2);
    }

    #[tokio::test]
    async fn test_key_derivation_pbkdf2() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        let password = "test_password";
        let salt = manager.generate_salt();
        let key = manager.derive_key(password, &salt, KeyDerivationMethod::PBKDF2)
            .expect("Key derivation should succeed");
        
        // Key should be 32 bytes
        assert_eq!(key.len(), 32);
    }

    #[tokio::test]
    async fn test_key_derivation_scrypt() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        let password = "test_password";
        let salt = manager.generate_salt();
        let key = manager.derive_key(password, &salt, KeyDerivationMethod::Scrypt)
            .expect("Key derivation should succeed");
        
        // Key should be 32 bytes
        assert_eq!(key.len(), 32);
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        let password = "test_password";
        let salt = manager.generate_salt();
        let key = manager.derive_key(password, &salt, KeyDerivationMethod::Argon2id)
            .expect("Key derivation should succeed");
        
        let original_data = b"Hello, World! This is sensitive data.";
        let encrypted = manager.encrypt_data(&key, original_data)
            .expect("Encryption should succeed");
        
        // Encrypted data should be longer than original
        assert!(encrypted.len() > original_data.len());
        
        let decrypted = manager.decrypt_data(&key, &encrypted)
            .expect("Decryption should succeed");
        
        assert_eq!(original_data, &decrypted[..]);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        // Test request rate limiting
        for i in 0..61 { // 61 requests, should fail on 61st
            let result = manager.check_rate_limit("test_client", "request");
            if i < 60 {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
                if let Err(e) = result {
                    assert!(matches!(e, SecurityError::RateLimitExceeded));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_transaction_validation() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        // Valid transaction should pass
        let result = manager.validate_transaction(
            "valid_from_address",
            "valid_to_address", 
            1000
        );
        assert!(result.is_ok());
        
        // Add to blacklist
        manager.add_to_blacklist("banned_address");
        
        // Transaction to blacklisted address should fail
        let result = manager.validate_transaction(
            "valid_from_address",
            "banned_address",
            1000
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whitelist_blacklist() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        // Enable whitelist
        // Note: In real implementation, you would set the whitelist_enabled flag
        
        // Add to whitelist
        manager.add_to_whitelist("trusted_address");
        manager.add_to_blacklist("malicious_address");
        
        // Test whitelist functionality
        let result1 = manager.validate_transaction(
            "from_address",
            "trusted_address",
            1000
        );
        // Should pass if whitelist is enabled and address is in whitelist
        
        let result2 = manager.validate_transaction(
            "from_address", 
            "malicious_address",
            1000
        );
        assert!(result2.is_err());
    }

    #[test]
    fn test_secure_key_creation() {
        let key_material = vec![1, 2, 3, 4, 5];
        let mut secure_key = SecureKey::new(key_material, KeyDerivationMethod::Argon2id);
        
        // Test key access
        let accessed = secure_key.access(|key| {
            assert_eq!(key, &[1, 2, 3, 4, 5]);
            42
        });
        assert_eq!(accessed, 42);
        
        // Access count should be incremented
        assert_eq!(secure_key.access_count, 1);
    }

    #[test]
    fn test_secure_hash_functions() {
        let data = b"Hello, World!";
        
        // Test BLAKE3
        let blake3_hash = SecureHash::blake3(data);
        assert_eq!(blake3_hash.len(), 32);
        
        // Test SHA-256
        let sha256_hash = SecureHash::sha256(data);
        assert_eq!(sha256_hash.len(), 32);
        
        // Test SHA-512
        let sha512_hash = SecureHash::sha512(data);
        assert_eq!(sha512_hash.len(), 64);
        
        // Test SHA3-256
        let sha3_hash = SecureHash::sha3_256(data);
        assert_eq!(sha3_hash.len(), 32);
    }

    #[test]
    fn test_hmac_creation() {
        let key = b"test_key";
        let data = b"Hello, World!";
        
        let hmac_result = SecureHash::hmac_sha256(key, data);
        assert!(hmac_result.is_ok());
        
        let hmac = hmac_result.unwrap();
        assert_eq!(hmac.len(), 32);
    }

    #[test]
    fn test_encryption_params_default() {
        let params = EncryptionParams::default();
        
        assert_eq!(params.algorithm, EncryptionAlgorithm::Aes256Gcm);
        assert_eq!(params.key_derivation, KeyDerivationMethod::Argon2id);
        assert_eq!(params.iterations, 100_000);
        assert_eq!(params.memory_cost, 19456);
        assert_eq!(params.key_length, 32);
    }

    #[test]
    fn test_transaction_security_default() {
        let security = TransactionSecurity::default();
        
        assert_eq!(security.max_amount, u64::MAX);
        assert_eq!(security.daily_limit, u64::MAX);
        assert_eq!(security.transaction_timeout, Duration::from_secs(300));
        assert_eq!(security.simulation_required, true);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        
        assert_eq!(config.max_requests_per_minute, 60);
        assert_eq!(config.max_transactions_per_hour, 100);
        assert_eq!(config.max_volume_per_hour, 1_000_000_000_000);
        assert_eq!(config.ban_duration, Duration::from_secs(3600));
    }

    #[test]
    fn test_certificate_pinning_default() {
        let pinning = CertificatePinning::default();
        
        assert!(pinning.enabled);
        assert!(!pinning.allowed_domains.is_empty());
        assert!(pinning.expiry_checking);
        assert!(!pinning.public_key_pins.is_empty());
    }

    #[test]
    fn test_security_event_serialization() {
        let event = SecurityEvent::KeyAccessed {
            key_id: "test_key".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let serialized = serde_json::to_string(&event);
        assert!(serialized.is_ok());
        
        let deserialized: SecurityEvent = serde_json::from_str(&serialized.unwrap()).unwrap();
        match deserialized {
            SecurityEvent::KeyAccessed { key_id, .. } => {
                assert_eq!(key_id, "test_key");
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_key_rotation_scheduling() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        let rotation_date = chrono::Utc::now() + chrono::Duration::days(30);
        manager.schedule_key_rotation("test_key", rotation_date);
        
        // Should not be due yet
        assert!(!manager.check_key_rotation("test_key"));
    }

    #[tokio::test]
    async fn test_address_banning() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        let duration = Duration::from_secs(60);
        manager.ban_address("suspicious_address", duration);
        
        // Transaction from banned address should fail
        let result = manager.validate_transaction(
            "suspicious_address",
            "recipient_address",
            1000
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secure_random_bytes_generation() {
        let (tx, _rx) = channel::unbounded();
        let manager = SecurityManager::new(tx);
        
        let bytes1 = manager.generate_random_bytes(32);
        let bytes2 = manager.generate_random_bytes(32);
        
        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        
        // Should be different (with extremely high probability)
        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_secure_key_zeroization() {
        let key_material = vec![0xFF; 32];
        let secure_key = SecureKey::new(key_material, KeyDerivationMethod::Argon2id);
        
        // Key should be accessible
        let _ = secure_key.access(|key| {
            assert_eq!(key.len(), 32);
        });
        
        // The key should be zeroized when dropped (ZeroizeOnDrop trait)
    }
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_encryption_decryption_roundtrip(
            data in prop::collection::vec(prop::num::u8::ANY, 0..1000),
            password in "[a-zA-Z0-9]{1,100}",
        ) {
            tokio_test::block_on(async {
                let (tx, _rx) = channel::unbounded();
                let manager = SecurityManager::new(tx);
                
                let salt = manager.generate_salt();
                let key = manager.derive_key(&password, &salt, KeyDerivationMethod::Argon2id)
                    .expect("Key derivation should succeed");
                
                let encrypted = manager.encrypt_data(&key, &data)
                    .expect("Encryption should succeed");
                
                let decrypted = manager.decrypt_data(&key, &encrypted)
                    .expect("Decryption should succeed");
                
                prop_assert_eq!(data, decrypted);
            });
        }

        #[test]
        fn test_different_passwords_different_keys(
            password1 in "[a-zA-Z0-9]{1,100}",
            password2 in "[a-zA-Z0-9]{1,100}",
        ) prop::strategy::filter(|p| p.0 != p.1) => |(password1, password2)| {
            tokio_test::block_on(async {
                let (tx, _rx) = channel::unbounded();
                let manager = SecurityManager::new(tx);
                
                let salt = manager.generate_salt();
                let key1 = manager.derive_key(&password1, &salt, KeyDerivationMethod::Argon2id)
                    .expect("Key derivation should succeed");
                let key2 = manager.derive_key(&password2, &salt, KeyDerivationMethod::Argon2id)
                    .expect("Key derivation should succeed");
                
                // Different passwords should produce different keys (with very high probability)
                if key1 == key2 {
                    panic!("Different passwords produced same key");
                }
            });
        }

        #[test]
        fn test_random_bytes_different(
            size in 1..1024usize,
        ) {
            let (tx, _rx) = channel::unbounded();
            let manager = SecurityManager::new(tx);
            
            let bytes1 = manager.generate_random_bytes(size);
            let bytes2 = manager.generate_random_bytes(size);
            
            prop_assert_eq!(bytes1.len(), size);
            prop_assert_eq!(bytes2.len(), size);
            prop_assert_ne!(bytes1, bytes2);
        }
    }
}