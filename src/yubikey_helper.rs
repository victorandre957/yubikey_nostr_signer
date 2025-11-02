use anyhow::{Context, Result, anyhow};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use nostr::prelude::*;
use std::sync::Mutex;

use crate::blob_operations;
use crate::credential::get_credential_id;
use crate::device::{find_fido_device, is_supported};

pub struct YubikeyKeyManager {
    device: Mutex<FidoKeyHid>,
    credential_id: Vec<u8>,
    selected_entry_index: usize,
    cached_public_key: PublicKey,
}

impl YubikeyKeyManager {
    pub fn new() -> Result<Self> {
        println!("🔑 Initializing YubiKey...");

        let mut device =
            find_fido_device().context("YubiKey not found. Connect the device and try again.")?;

        if !is_supported(&device)? {
            return Err(anyhow!("This device does not support largeBlob"));
        }

        let credential_id =
            get_credential_id(&mut device).context("Failed to configure credential")?;

        println!("✅ YubiKey configured successfully\n");

        let (selected_entry_index, key_data) =
            blob_operations::select_and_read_entry(&mut device, &credential_id)
                .context("Failed to select entry")?;

        println!("\n🔍 Validating selected key...");
        let key_hex = String::from_utf8(key_data).context("Invalid key data")?;

        let keys = Keys::parse(&key_hex).context("Failed to parse private key")?;

        let cached_public_key = keys.public_key();

        drop(keys);

        println!("✅ Valid key!");
        println!("   Pubkey: {}\n", cached_public_key.to_bech32()?);

        Ok(Self {
            device: Mutex::new(device),
            credential_id,
            selected_entry_index,
            cached_public_key,
        })
    }

    pub fn get_public_key(&self) -> Result<PublicKey> {
        Ok(self.cached_public_key)
    }

    pub fn load_private_key(&self) -> Result<Keys> {
        println!("🔐 Loading key from YubiKey for signing...");

        let mut device = self
            .device
            .lock()
            .map_err(|_| anyhow!("Failed to access device"))?;

        let key_data = blob_operations::read_blob_entry_by_index(
            &mut device,
            &self.credential_id,
            self.selected_entry_index,
        )
        .context("Failed to read entry from YubiKey")?;

        let key_hex = String::from_utf8(key_data).context("Invalid key data")?;

        let keys = Keys::parse(&key_hex).context("Failed to parse private key")?;

        println!("✅ Key loaded (will be discarded after use)\n");

        Ok(keys)
    }

    pub fn with_key<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&Keys) -> Result<R>,
    {
        let keys = self.load_private_key()?;
        let result = operation(&keys);
        drop(keys);
        println!("🧹 Key removed from memory\n");
        result
    }
}
