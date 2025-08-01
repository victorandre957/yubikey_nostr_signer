use anyhow::{anyhow, Context, Result};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use std::io::{self, Write};
use crate::auth::get_pin_from_user;

pub fn write_blob(device: &mut FidoKeyHid, _credential_id: &[u8], data: &str) -> Result<()> {
    println!("Iniciando escrita no largeBlob...");
    let pin = get_pin_from_user()?;
    
    let new_data_hex = hex::encode(data.as_bytes());
    println!("Dados em hex: {}", new_data_hex);
    
    let existing_result = device.get_large_blob();
    let mut combined_data = Vec::new();
    
    if let Ok(existing) = existing_result {
        if !existing.large_blob_array.is_empty() {
            if let Ok(existing_str) = String::from_utf8(existing.large_blob_array.clone()) {
                if existing_str == hex::encode("EMPTY") {
                    combined_data = Vec::new();
                } else {
                    let max_size = 1024;
                    let needed_space = existing.large_blob_array.len() + new_data_hex.len() + 1;
                    
                    if needed_space > max_size {
                        println!("Não há espaço suficiente no largeBlob!");
                        println!("Espaço usado: {} bytes", existing.large_blob_array.len());
                        println!("Espaço necessário: {} bytes", needed_space);
                        println!("Espaço máximo: {} bytes", max_size);
                        
                        // Show existing data
                        let entries: Vec<&str> = existing_str.split('|').collect();
                        println!("\nDados existentes:");
                        for (i, entry) in entries.iter().enumerate() {
                            if !entry.is_empty() {
                                if let Ok(decoded_bytes) = hex::decode(entry) {
                                    if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                                        println!("{}: {} (hex: {})", i + 1, decoded_str, entry);
                                    }
                                }
                            }
                        }
                        
                        print!("Digite o número da entrada para remover (ou 0 para cancelar): ");
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        let choice: usize = input.trim().parse().unwrap_or(0);
                        
                        if choice == 0 {
                            println!("Operação cancelada.");
                            return Ok(());
                        } else if choice > 0 && choice <= entries.len() {
                            let mut new_entries = entries;
                            new_entries.remove(choice - 1);
                            let filtered_data = new_entries.join("|");
                            combined_data = filtered_data.into_bytes();
                            println!("Entrada {} removida.", choice);
                        } else {
                            println!("Número inválido. Operação cancelada.");
                            return Ok(());
                        }
                    } else {
                        combined_data = existing.large_blob_array;
                    }
                }
            } else {
                return Err(anyhow!("Não foi possível decodificar os dados existentes."));
            }
        }
    }
    
    if !combined_data.is_empty() {
        combined_data.push(b'|');
    }
    combined_data.extend_from_slice(new_data_hex.as_bytes());
    
    let _result = device.write_large_blob(Some(&pin), combined_data)
        .context("Falha ao escrever no largeBlob. Tente novamente ou verifique se a chave está desbloqueada.")?;

    println!("Sucesso! Dados escritos no largeBlob.");
    Ok(())
}

pub fn read_blob(device: &mut FidoKeyHid, _credential_id: &[u8]) -> Result<()> {
    println!("Iniciando leitura do largeBlob...");
    
    let result = device.get_large_blob()
        .context("Falha ao ler do largeBlob.")?;
    
    if result.large_blob_array.is_empty() {
        println!("O largeBlob está vazio.");
    } else {
        println!("Sucesso! Dados lidos do largeBlob:");
        println!("Tamanho total: {} bytes", result.large_blob_array.len());
        
        if let Ok(blob_content) = String::from_utf8(result.large_blob_array.clone()) {
            if blob_content == hex::encode("EMPTY") {
                println!("O largeBlob foi esvaziado (contém placeholder vazio).");
                return Ok(());
            }
            
            let entries: Vec<&str> = blob_content.split('|').collect();
            let non_empty_entries: Vec<&str> = entries.iter().filter(|e| !e.is_empty()).cloned().collect();
            println!("Total de entradas: {}", non_empty_entries.len());
            
            for (i, entry) in non_empty_entries.iter().enumerate() {
                match hex::decode(entry) {
                    Ok(decoded_bytes) => {
                        match String::from_utf8(decoded_bytes) {
                            Ok(decoded_str) => {
                                println!("Entrada {}: \"{}\"", i + 1, decoded_str);
                                println!("  Hex: {}", entry);
                            }
                            Err(_) => {
                                println!("Entrada {}: (dados binários)", i + 1);
                                println!("  Hex: {}", entry);
                            }
                        }
                    }
                    Err(_) => {
                        println!("Entrada {}: (formato hex inválido) \"{}\"", i + 1, entry);
                    }
                }
            }
        } else {
            println!("Conteúdo (bytes brutos): {:?}", result.large_blob_array);
        }
    }
    Ok(())
}

pub fn delete_single_entry(device: &mut FidoKeyHid, _credential_id: &[u8]) -> Result<()> {
    println!("Lendo entradas existentes...");
    
    let current_blob = match device.get_large_blob() {
        Ok(response) => {
            if response.large_blob_array.is_empty() {
                println!("O largeBlob está vazio. Nada para apagar.");
                return Ok(());
            }
            response.large_blob_array
        },
        Err(e) => return Err(anyhow!("Erro ao ler blob: {}", e)),
    };

    let blob_content = String::from_utf8(current_blob)
        .context("Não foi possível decodificar os dados existentes.")?;
    
    if blob_content == hex::encode("EMPTY") {
        println!("O largeBlob está vazio (contém placeholder). Nada para apagar.");
        return Ok(());
    }
    
    let entries: Vec<&str> = blob_content.split('|').collect();
    let non_empty_entries: Vec<&str> = entries.iter().filter(|e| !e.is_empty()).cloned().collect();
    
    if non_empty_entries.is_empty() {
        println!("Não há entradas para apagar.");
        return Ok(());
    }
    
    println!("\nEntradas existentes:");
    for (i, entry) in non_empty_entries.iter().enumerate() {
        match hex::decode(entry) {
            Ok(decoded_bytes) => {
                match String::from_utf8(decoded_bytes) {
                    Ok(decoded_str) => {
                        println!("{}: \"{}\"", i + 1, decoded_str);
                    }
                    Err(_) => {
                        println!("{}: (dados binários)", i + 1);
                    }
                }
            }
            Err(_) => {
                println!("{}: (formato hex inválido)", i + 1);
            }
        }
    }
    
    print!("Digite o número da entrada para remover (ou 0 para cancelar): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let key_index: usize = input.trim().parse().unwrap_or(0);
    
    if key_index == 0 || key_index > non_empty_entries.len() {
        println!("Operação cancelada.");
        return Ok(());
    }
    
    let mut updated_entries = non_empty_entries;
    updated_entries.remove(key_index - 1);

    let pin = get_pin_from_user()?;
    
    if updated_entries.is_empty() {
        println!("Esvaziando largeBlob completamente...");
        let empty_placeholder = hex::encode("EMPTY").into_bytes();
        
        println!("Escrevendo placeholder vazio... Toque na chave se piscar.");
        
        match device.write_large_blob(Some(&pin), empty_placeholder.clone()) {
            Ok(_) => {
                println!("LargeBlob esvaziado com sucesso!");
            },
            Err(e) => {
                println!("Primeira tentativa falhou: {}. Tentando novamente...", e);
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                match device.write_large_blob(Some(&pin), empty_placeholder) {
                    Ok(_) => {
                        println!("LargeBlob esvaziado com sucesso na segunda tentativa!");
                    },
                    Err(e2) => {
                        return Err(anyhow!("Falha ao esvaziar o largeBlob após duas tentativas: {}. Tente novamente ou verifique se a chave está desbloqueada.", e2));
                    }
                }
            }
        }
    } else {
        let data = updated_entries.join("|").into_bytes();
        
        println!("Escrevendo dados atualizados... Toque na chave se piscar.");
        match device.write_large_blob(Some(&pin), data.clone()) {
            Ok(_) => {
                println!("Entrada {} removida com sucesso!", key_index);
            },
            Err(e) => {
                // Sometimes FIDO2 devices need a moment between operations
                println!("Primeira tentativa falhou: {}. Tentando novamente...", e);
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                match device.write_large_blob(Some(&pin), data) {
                    Ok(_) => {
                        println!("Entrada {} removida com sucesso na segunda tentativa!", key_index);
                    },
                    Err(e2) => {
                        return Err(anyhow!("Falha ao atualizar o largeBlob após duas tentativas: {}. Tente novamente ou verifique se a chave está desbloqueada.", e2));
                    }
                }
            }
        }
    }
    drop(pin);
    
    Ok(())
}
