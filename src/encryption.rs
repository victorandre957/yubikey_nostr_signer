use anyhow::{anyhow, Context, Result};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use rand::Rng;
use crate::credential::get_hmac_secret;

/// Encrypts plaintext data using AES-256-GCM with a key derived from HMAC secret
pub fn encrypt_data(device: &mut FidoKeyHid, credential_id: &[u8], plaintext: &str) -> Result<Vec<u8>> {
    let mut salt = [0u8; 32];
    rand::rng().fill(&mut salt);
    
    let mut hmac_secret = get_hmac_secret(device, credential_id, &salt)
        .context("Failed to get encryption key")?;
    
    // Generate a 12-byte nonce for GCM (recommended size)
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(&hmac_secret)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
    
    // Encrypt the plaintext (GCM handles authentication automatically)
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;
    
    // Construct result: salt (32) + nonce (12) + ciphertext_with_tag
    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    // Clear sensitive data
    hmac_secret.fill(0);
    
    Ok(result)
}

/// Decrypts encrypted data using AES-256-GCM with a key derived from HMAC secret
pub fn decrypt_data(device: &mut FidoKeyHid, credential_id: &[u8], encrypted_data: &[u8]) -> Result<String> {
    if encrypted_data.len() < 44 { // 32 (salt) + 12 (nonce) + minimum ciphertext
        return Err(anyhow!("Invalid encrypted data"));
    }
    
    let salt: [u8; 32] = encrypted_data[0..32].try_into()
        .map_err(|_| anyhow!("Error extracting decryption data"))?;
    let nonce_bytes: [u8; 12] = encrypted_data[32..44].try_into()
        .map_err(|_| anyhow!("Error extracting decryption data"))?;
    let ciphertext = &encrypted_data[44..];
    
    let mut hmac_secret = get_hmac_secret(device, credential_id, &salt)
        .context("Error extracting decryption Key")?;
    
    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(&hmac_secret)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
    
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Decrypt and authenticate
    let plaintext_bytes = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;
    
    let result = String::from_utf8(plaintext_bytes)
        .context("Invalid decrypted data")?;
    
    // Clear sensitive data
    hmac_secret.fill(0);
    
    Ok(result)
}
