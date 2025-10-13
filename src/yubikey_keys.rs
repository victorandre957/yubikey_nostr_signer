use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use nostr::prelude::*;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::credential::get_credential_id;
use crate::device::{find_fido_device, is_supported};
use crate::encryption::decrypt_data;

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

        // Obtém o conteúdo do blob
        let result = device
            .get_large_blob()
            .context("Falha ao ler largeBlob")?;

        if result.large_blob_array.is_empty() {
            return Err(anyhow!("O largeBlob está vazio. Use 'cargo run' para adicionar chaves."));
        }

        let blob_content = String::from_utf8(result.large_blob_array)
            .context("Dados inválidos no largeBlob")?;

        if blob_content == general_purpose::STANDARD.encode("EMPTY") {
            return Err(anyhow!("O largeBlob está vazio. Use 'cargo run' para adicionar chaves."));
        }

        // Parse das entradas
        let entries: Vec<String> = blob_content
            .split('|')
            .filter(|e| !e.is_empty())
            .map(|e| e.to_string())
            .collect();

        if entries.is_empty() {
            return Err(anyhow!("Nenhuma entrada encontrada no largeBlob"));
        }

        // Exibe as entradas disponíveis
        println!("📋 Entradas disponíveis:");
        for (i, entry) in entries.iter().enumerate() {
            if let Some(colon_pos) = entry.find(':') {
                let entry_id = &entry[..colon_pos];
                println!("   {}. {}", i + 1, entry_id);
            } else {
                println!("   {}. (entrada sem ID)", i + 1);
            }
        }

        // Solicita escolha do usuário
        print!("\n🔑 Escolha qual entrada usar para o bunker (1-{}): ", entries.len());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice: usize = input.trim().parse()
            .context("Entrada inválida")?;

        if choice == 0 || choice > entries.len() {
            return Err(anyhow!("Escolha inválida"));
        }

        let selected_entry_index = choice - 1;
        let selected_entry = &entries[selected_entry_index];

        // Carrega a chave UMA VEZ para obter a chave pública e validar
        println!("\n🔐 Validando chave selecionada...");
        let (cached_public_key, _) = Self::decrypt_and_parse_entry(&mut device, &credential_id, selected_entry)?;
        
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

        // Obtém o conteúdo do blob
        let blob_content = self.get_blob_content(&mut device)?;
        
        // Parse das entradas
        let entries = self.parse_blob_entries(&blob_content);
        
        if self.selected_entry_index >= entries.len() {
            return Err(anyhow!("Entrada selecionada não existe mais"));
        }

        // Usa a entrada pré-selecionada
        let selected_entry = &entries[self.selected_entry_index];
        
        // Descriptografa e parseia a entrada
        let (_, keys) = Self::decrypt_and_parse_entry(&mut device, &self.credential_id, selected_entry)?;

        println!("✅ Chave carregada (será descartada após uso)\n");

        Ok(keys)
    }

    /// Descriptografa e parseia uma entrada, retornando a chave pública e as Keys completas
    fn decrypt_and_parse_entry(device: &mut FidoKeyHid, credential_id: &[u8], entry: &str) -> Result<(PublicKey, Keys)> {
        let key_data = if let Some(colon_pos) = entry.find(':') {
            let encrypted_base64 = &entry[colon_pos + 1..];
            let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_base64)
                .context("Falha ao decodificar base64")?;
            decrypt_data(device, credential_id, &encrypted_bytes)?
        } else {
            // Formato antigo sem ID - tenta base64
            if let Ok(encrypted_bytes) = general_purpose::STANDARD.decode(entry) {
                decrypt_data(device, credential_id, &encrypted_bytes)?
            } else if let Ok(encrypted_bytes) = hex::decode(entry) {
                // Fallback para hex
                decrypt_data(device, credential_id, &encrypted_bytes)?
            } else {
                return Err(anyhow!("Formato de entrada inválido"));
            }
        };

        let keys = Keys::parse(&key_data)
            .context("Falha ao parsear chave privada")?;

        let public_key = keys.public_key();

        Ok((public_key, keys))
    }

    /// Obtém o conteúdo do blob da YubiKey
    fn get_blob_content(&self, device: &mut FidoKeyHid) -> Result<String> {
        let result = device
            .get_large_blob()
            .context("Falha ao ler largeBlob")?;

        if result.large_blob_array.is_empty() {
            return Err(anyhow!("O largeBlob está vazio"));
        }

        // Verifica se é o placeholder "EMPTY"
        if let Ok(content) = String::from_utf8(result.large_blob_array.clone()) {
            if content == general_purpose::STANDARD.encode("EMPTY") {
                return Err(anyhow!("O largeBlob está vazio"));
            }
        }

        String::from_utf8(result.large_blob_array)
            .context("Dados inválidos no largeBlob")
    }

    /// Faz parse das entradas do blob
    fn parse_blob_entries(&self, blob_content: &str) -> Vec<String> {
        if blob_content == general_purpose::STANDARD.encode("EMPTY") {
            return Vec::new();
        }

        blob_content
            .split('|')
            .filter(|e| !e.is_empty())
            .map(|e| e.to_string())
            .collect()
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

/// Versão thread-safe do gerenciador para uso no bunker
pub type SharedYubikeyManager = Arc<YubikeyKeyManager>;
