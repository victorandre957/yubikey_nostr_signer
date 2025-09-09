use anyhow::{anyhow, Context, Result};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::Rng;
use crate::credential::get_hmac_secret;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

/// Encrypts plaintext data using AES-256-CBC with a key derived from HMAC secret
pub fn encrypt_data(device: &mut FidoKeyHid, credential_id: &[u8], plaintext: &str) -> Result<Vec<u8>> {
    let mut salt = [0u8; 32];
    rand::rng().fill(&mut salt);
    
    let mut hmac_secret = get_hmac_secret(device, credential_id, &salt)
        .context("Failed to get encryption key")?;
    
    let mut iv = [0u8; 16];
    rand::rng().fill(&mut iv);
    
    let mut buffer = plaintext.as_bytes().to_vec();
    let original_len = buffer.len();
    let padding_len = 16 - (buffer.len() % 16);
    buffer.extend(vec![padding_len as u8; padding_len]);
    
    let cipher = Aes256CbcEnc::new(&hmac_secret.into(), &iv.into());
    let ciphertext = cipher.encrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut buffer, original_len)
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;
    
    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&iv);
    result.extend_from_slice(ciphertext);
    
    hmac_secret.fill(0);
    buffer.fill(0);
    
    Ok(result)
}

/// Decrypts encrypted data using AES-256-CBC with a key derived from HMAC secret
pub fn decrypt_data(device: &mut FidoKeyHid, credential_id: &[u8], encrypted_data: &[u8]) -> Result<String> {
    if encrypted_data.len() < 48 {
        return Err(anyhow!("Invalid encrypted data"));
    }
    
    let salt: [u8; 32] = encrypted_data[0..32].try_into()
        .map_err(|_| anyhow!("Error extracting decryption data"))?;
    let iv: [u8; 16] = encrypted_data[32..48].try_into()
        .map_err(|_| anyhow!("Error extracting decryption data"))?;
    let ciphertext = &encrypted_data[48..];
    
    let mut hmac_secret = get_hmac_secret(device, credential_id, &salt)
        .context("Error extracting decryption Key")?;
    
    let cipher = Aes256CbcDec::new(&hmac_secret.into(), &iv.into());
    let mut buffer = ciphertext.to_vec();
    
    let plaintext = cipher.decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut buffer)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;
    
    let result = String::from_utf8(plaintext.to_vec())
        .context("Invalid decrypted data")?;
    
    hmac_secret.fill(0);
    buffer.fill(0);
    
    Ok(result)
}
