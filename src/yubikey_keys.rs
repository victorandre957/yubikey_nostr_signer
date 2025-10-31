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
        println!("🔑 Inicializando YubiKey...");

        let mut device = find_fido_device()
            .context("YubiKey não encontrada. Conecte o dispositivo e tente novamente.")?;

        if !is_supported(&device)? {
            return Err(anyhow!("Este dispositivo não suporta largeBlob"));
        }

        let credential_id =
            get_credential_id(&mut device).context("Falha ao configurar credencial")?;

        println!("✅ YubiKey configurada com sucesso\n");

        let (selected_entry_index, key_data) =
            blob_operations::select_and_read_entry(&mut device, &credential_id)
                .context("Falha ao selecionar entrada")?;

        println!("\n🔍 Validando chave selecionada...");
        let key_hex = String::from_utf8(key_data).context("Dados da chave inválidos")?;

        let keys = Keys::parse(&key_hex).context("Falha ao parsear chave privada")?;

        let cached_public_key = keys.public_key();

        drop(keys);

        println!("✅ Chave válida!");
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
        println!("🔐 Carregando chave da YubiKey para assinatura...");

        let mut device = self
            .device
            .lock()
            .map_err(|_| anyhow!("Falha ao acessar dispositivo"))?;

        let key_data = blob_operations::read_blob_entry_by_index(
            &mut device,
            &self.credential_id,
            self.selected_entry_index,
        )
        .context("Falha ao ler entrada da YubiKey")?;

        let key_hex = String::from_utf8(key_data).context("Dados da chave inválidos")?;

        let keys = Keys::parse(&key_hex).context("Falha ao parsear chave privada")?;

        println!("✅ Chave carregada (será descartada após uso)\n");

        Ok(keys)
    }

    pub fn with_key<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&Keys) -> Result<R>,
    {
        let keys = self.load_private_key()?;
        let result = operation(&keys);
        drop(keys);
        println!("🧹 Chave removida da memória\n");
        result
    }
}
