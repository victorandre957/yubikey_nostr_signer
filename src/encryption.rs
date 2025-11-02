use crate::credential::get_hmac_secret;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use anyhow::{Context, Result, anyhow};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use rand::Rng;
use zeroize::Zeroize;

pub fn encrypt_data(
    device: &mut FidoKeyHid,
    credential_id: &[u8],
    plaintext: &str,
) -> Result<Vec<u8>> {
    let mut salt = [0u8; 32];
    rand::rng().fill(&mut salt);

    let mut hmac_secret =
        get_hmac_secret(device, credential_id, &salt).context("Failed to get encryption key")?;

    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&hmac_secret)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    hmac_secret.zeroize();

    Ok(result)
}

pub fn decrypt_data(
    device: &mut FidoKeyHid,
    credential_id: &[u8],
    encrypted_data: &[u8],
) -> Result<String> {
    if encrypted_data.len() < 44 {
        return Err(anyhow!("Invalid encrypted data"));
    }

    let salt: [u8; 32] = encrypted_data[0..32]
        .try_into()
        .map_err(|_| anyhow!("Error extracting decryption data"))?;
    let nonce_bytes: [u8; 12] = encrypted_data[32..44]
        .try_into()
        .map_err(|_| anyhow!("Error extracting decryption data"))?;
    let ciphertext = &encrypted_data[44..];

    let mut hmac_secret =
        get_hmac_secret(device, credential_id, &salt).context("Error extracting decryption Key")?;

    let cipher = Aes256Gcm::new_from_slice(&hmac_secret)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;

    let result = String::from_utf8(plaintext_bytes).context("Invalid decrypted data")?;

    hmac_secret.zeroize();

    Ok(result)
}
