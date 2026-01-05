//! Cryptographic utilities for secure token storage.
//!
//! This module provides encryption and decryption functions for sensitive tokens
//! stored in the database using AES-256-GCM authenticated encryption.

use aes_gcm::{
    aead::{generic_array::typenum::U12, Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use log::debug;
use std::env;

/// The length of the nonce in bytes (96 bits for AES-GCM)
const NONCE_LENGTH: usize = 12;

/// Gets the encryption key from environment variable.
///
/// The key must be exactly 32 bytes (256 bits) encoded as a 64-character hex string.
///
/// # Returns
///
/// - `Ok([u8; 32])`: The 32-byte encryption key
/// - `Err`: If the key is missing, invalid hex, or wrong length
fn get_encryption_key() -> Result<[u8; 32], Box<dyn std::error::Error + Send + Sync>> {
    let key_hex = env::var("TOKEN_ENCRYPTION_KEY").map_err(|_| {
        "TOKEN_ENCRYPTION_KEY environment variable is not set. Generate a 32-byte key with: openssl rand -hex 32"
    })?;

    let key_bytes = hex::decode(&key_hex).map_err(|e| {
        format!(
            "TOKEN_ENCRYPTION_KEY is not valid hex: {}. Generate a key with: openssl rand -hex 32",
            e
        )
    })?;

    if key_bytes.len() != 32 {
        return Err(format!(
            "TOKEN_ENCRYPTION_KEY must be exactly 32 bytes (64 hex chars), got {} bytes",
            key_bytes.len()
        )
        .into());
    }

    let mut key: [u8; 32] = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

/// Encrypts a token using AES-256-GCM.
///
/// The function generates a random nonce and prepends it to the ciphertext.
/// The output format is: nonce (12 bytes) || ciphertext || auth_tag
///
/// # Parameters
///
/// - `plaintext`: The token to encrypt
///
/// # Returns
///
/// - `Ok(String)`: The hex-encoded encrypted token (nonce + ciphertext)
/// - `Err`: If encryption fails or the key is not configured
pub fn encrypt_token(plaintext: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    // Generate a random nonce
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate random nonce: {}", e))?;
    let nonce: Nonce<U12> = nonce_bytes.into();

    // Encrypt the plaintext
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext and encode as hex
    let mut result = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    debug!("Token encrypted successfully");
    Ok(hex::encode(result))
}

/// Decrypts a token that was encrypted with `encrypt_token`.
///
/// Expects the input to be a hex-encoded string containing: nonce (12 bytes) || ciphertext || auth_tag
///
/// # Parameters
///
/// - `encrypted_hex`: The hex-encoded encrypted token
///
/// # Returns
///
/// - `Ok(String)`: The decrypted token
/// - `Err`: If decryption fails, the key is wrong, or the data is corrupted
pub fn decrypt_token(
    encrypted_hex: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    // Decode from hex
    let encrypted_bytes =
        hex::decode(encrypted_hex).map_err(|e| format!("Invalid hex in encrypted token: {}", e))?;

    if encrypted_bytes.len() < NONCE_LENGTH {
        return Err("Encrypted token is too short".into());
    }

    // Extract nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(NONCE_LENGTH);
    let nonce_array: [u8; NONCE_LENGTH] =
        nonce_bytes.try_into().map_err(|_| "Invalid nonce length")?;
    let nonce: Nonce<U12> = nonce_array.into();

    // Decrypt
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| "Decryption failed - wrong key or corrupted data")?;

    let token = String::from_utf8(plaintext)
        .map_err(|e| format!("Decrypted token is not valid UTF-8: {}", e))?;

    debug!("Token decrypted successfully");
    Ok(token)
}

/// Checks if token encryption is configured.
///
/// # Returns
///
/// `true` if TOKEN_ENCRYPTION_KEY is set, `false` otherwise
pub fn is_encryption_configured() -> bool {
    env::var("TOKEN_ENCRYPTION_KEY").is_ok()
}

/// Validates that encryption is properly configured.
///
/// This function should be called at application startup to ensure
/// that token encryption is properly configured before processing any tokens.
///
/// # Returns
///
/// - `Ok(())`: If encryption is properly configured
/// - `Err`: If TOKEN_ENCRYPTION_KEY is missing or invalid
///
/// # Errors
///
/// Returns an error if:
/// - TOKEN_ENCRYPTION_KEY environment variable is not set
/// - The key is not valid hexadecimal
/// - The key is not exactly 32 bytes (64 hex characters)
pub fn validate_encryption_config() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    get_encryption_key()?;
    debug!("Token encryption configuration validated successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to prevent parallel test execution that manipulates TOKEN_ENCRYPTION_KEY
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Set a test key
        env::set_var(
            "TOKEN_ENCRYPTION_KEY",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );

        let original = "test_token_12345";
        let encrypted = encrypt_token(original).unwrap();

        // Encrypted should be different from original
        assert_ne!(encrypted, original);

        // Decrypt should recover original
        let decrypted = decrypt_token(&encrypted).unwrap();
        assert_eq!(decrypted, original);

        // Clean up
        env::remove_var("TOKEN_ENCRYPTION_KEY");
    }

    #[test]
    fn test_different_encryptions_produce_different_output() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var(
            "TOKEN_ENCRYPTION_KEY",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );

        let original = "test_token";
        let encrypted1 = encrypt_token(original).unwrap();
        let encrypted2 = encrypt_token(original).unwrap();

        // Due to random nonce, same plaintext should produce different ciphertext
        assert_ne!(encrypted1, encrypted2);

        // Both should decrypt to the same value
        assert_eq!(decrypt_token(&encrypted1).unwrap(), original);
        assert_eq!(decrypt_token(&encrypted2).unwrap(), original);

        env::remove_var("TOKEN_ENCRYPTION_KEY");
    }
}
