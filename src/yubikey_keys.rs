use anyhow::{Context, Result, anyhow};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use nostr::prelude::*;
use std::sync::Mutex;

use crate::blob_operations;
use crate::credential::get_credential_id;
use crate::device::{find_fido_device, is_supported};

/// Gerenciador de chaves da YubiKey que carrega chaves sob demanda
pub struct YubikeyKeyManager {
    device: Mutex<FidoKeyHid>,
    credential_id: Vec<u8>,
    /// Índice da entrada escolhida pelo usuário (0-based)
    selected_entry_index: usize,
    /// Cache da chave pública (para evitar leituras desnecessárias)
    cached_public_key: PublicKey,
}

impl YubikeyKeyManager {
    /// Inicializa o gerenciador e configura a YubiKey
    /// Solicita ao usuário escolher qual entrada usar (uma vez)
    pub fn new() -> Result<Self> {
        println!("🔑 Inicializando YubiKey...");
        
        let mut device = find_fido_device()
            .context("YubiKey não encontrada. Conecte o dispositivo e tente novamente.")?;
        
        if !is_supported(&device)? {
            return Err(anyhow!("Este dispositivo não suporta largeBlob"));
        }

        let credential_id = get_credential_id(&mut device)
            .context("Falha ao configurar credencial")?;

        println!("✅ YubiKey configurada com sucesso\n");

        // Usa a função de blob_operations para selecionar entrada
        let (selected_entry_index, key_data) = blob_operations::select_and_read_entry(&mut device, &credential_id)
            .context("Falha ao selecionar entrada")?;

        // Carrega a chave UMA VEZ para obter a chave pública e validar
        println!("\n� Validando chave selecionada...");
        let key_hex = String::from_utf8(key_data)
            .context("Dados da chave inválidos")?;
        
        let keys = Keys::parse(&key_hex)
            .context("Falha ao parsear chave privada")?;
        
        let cached_public_key = keys.public_key();
        
        // Limpa as keys da memória
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

    /// Retorna a chave pública (cached, sem acessar YubiKey)
    pub fn get_public_key(&self) -> Result<PublicKey> {
        Ok(self.cached_public_key)
    }

    /// Lê a chave privada da YubiKey SOB DEMANDA (requer PIN do usuário)
    /// Retorna a chave que deve ser usada imediatamente e descartada
    /// Esta função é chamada apenas quando realmente precisa assinar algo
    pub fn load_private_key(&self) -> Result<Keys> {
        println!("🔐 Carregando chave da YubiKey para assinatura...");
        
        let mut device = self.device.lock()
            .map_err(|_| anyhow!("Falha ao acessar dispositivo"))?;

        // Usa blob_operations para ler a entrada por índice
        let key_data = blob_operations::read_blob_entry_by_index(
            &mut device, 
            &self.credential_id, 
            self.selected_entry_index
        ).context("Falha ao ler entrada da YubiKey")?;

        let key_hex = String::from_utf8(key_data)
            .context("Dados da chave inválidos")?;

        let keys = Keys::parse(&key_hex)
            .context("Falha ao parsear chave privada")?;

        println!("✅ Chave carregada (será descartada após uso)\n");

        Ok(keys)
    }

    /// Carrega a chave, executa uma operação e limpa a memória
    /// Este é o método principal para usar a chave de forma segura
    pub fn with_key<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&Keys) -> Result<R>,
    {
        // Carrega a chave SOB DEMANDA da YubiKey
        let keys = self.load_private_key()?;
        
        // Executa a operação (ex: assinar evento)
        let result = operation(&keys);
        
        // Limpa a chave da memória
        drop(keys);
        
        println!("🧹 Chave removida da memória\n");
        
        result
    }
}
