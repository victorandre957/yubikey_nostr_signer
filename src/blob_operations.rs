use anyhow::{anyhow, Context, Result};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use std::io::{self, Write};
use crate::auth::get_pin_from_user;
use crate::encryption::{encrypt_data, decrypt_data};

pub fn write_blob(device: &mut FidoKeyHid, credential_id: &[u8], data: &str) -> Result<()> {
    print!("Enter an ID for this entry: ");
    io::stdout().flush()?;
    let mut id_input = String::new();
    io::stdin().read_line(&mut id_input)?;
    let entry_id = id_input.trim();
    
    if entry_id.is_empty() {
        println!("ID cannot be empty.");
        return Ok(());
    }
    
    println!("Encrypting data...");
    
    let encrypted_data = encrypt_data(device, credential_id, data)
        .context("Failed to encrypt data")?;
    
    // Format: "ID:encrypted_hex"
    let entry_with_id = format!("{}:{}", entry_id, hex::encode(&encrypted_data));
    
    let existing_result = device.get_large_blob();
    let mut combined_data = Vec::new();
    
    if let Ok(existing) = existing_result {
        if !existing.large_blob_array.is_empty() {
            if let Ok(existing_str) = String::from_utf8(existing.large_blob_array.clone()) {
                if existing_str == hex::encode("EMPTY") {
                    combined_data = Vec::new();
                } else {
                    let max_size = 1024;
                    let needed_space = existing.large_blob_array.len() + entry_with_id.len() + 1;
                    
                    if needed_space > max_size {
                        println!("Insufficient space ({}/{} bytes).", needed_space, max_size);
                        
                        let entries: Vec<&str> = existing_str.split('|').collect();
                        println!("\nExisting entries:");
                        for (i, entry) in entries.iter().enumerate() {
                            if !entry.is_empty() {
                                if let Some(colon_pos) = entry.find(':') {
                                    let entry_id = &entry[..colon_pos];
                                    println!("{}: {}", i + 1, entry_id);
                                } else {
                                    println!("{}: (legacy format without ID)", i + 1);
                                }
                            }
                        }
                        
                        print!("Enter the entry number to remove (or 0 to cancel): ");
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        let choice: usize = input.trim().parse().unwrap_or(0);
                        
                        if choice == 0 {
                            println!("Operation cancelled.");
                            return Ok(());
                        }
                        
                        let mut entries: Vec<String> = existing_str.split('|')
                            .filter(|e| !e.is_empty())
                            .map(|e| e.to_string())
                            .collect();
                        
                        if choice > 0 && choice <= entries.len() {
                            entries.remove(choice - 1);
                            println!("Entry {} removed.", choice);
                            
                            if entries.is_empty() {
                                combined_data = Vec::new();
                            } else {
                                combined_data = entries.join("|").into_bytes();
                            }
                        } else {
                            println!("Invalid choice.");
                            return Ok(());
                        }
                    } else {
                        combined_data = existing.large_blob_array;
                    }
                }
            } else {
                return Err(anyhow!("Invalid data in largeBlob"));
            }
        }
    }

    let final_data = if combined_data.is_empty() {
        entry_with_id.into_bytes()
    } else {
        let mut result = combined_data;
        result.push(b'|');
        result.extend_from_slice(entry_with_id.as_bytes());
        result
    };

    // Request PIN only when needed for the write operation
    let mut pin = get_pin_from_user()?;
    match device.write_large_blob(Some(pin.as_str()), final_data) {
        Ok(_) => {
            println!("✓ Data encrypted and stored successfully!");
        }
        Err(e) => {
            // Clear PIN from memory before returning error
            unsafe {
                let bytes = pin.as_bytes_mut();
                bytes.fill(0);
            }
            return Err(anyhow!("Error writing to largeBlob: {}", e));
        }
    }
    
    unsafe {
        let bytes = pin.as_bytes_mut();
        bytes.fill(0);
    }

    Ok(())
}

pub fn read_blob(device: &mut FidoKeyHid, credential_id: &[u8]) -> Result<()> {
    println!("Reading data...");
    
    let result = device.get_large_blob()
        .context("Failed to read from largeBlob.")?;
    
    if result.large_blob_array.is_empty() {
        println!("The largeBlob is empty.");
        return Ok(());
    }

    if let Ok(blob_content) = String::from_utf8(result.large_blob_array.clone()) {
        if blob_content == hex::encode("EMPTY") {
            println!("The largeBlob is empty.");
            return Ok(());
        }
        
        let entries: Vec<&str> = blob_content.split('|').collect();
        let non_empty_entries: Vec<&str> = entries.iter().filter(|e| !e.is_empty()).cloned().collect();
        
        if non_empty_entries.is_empty() {
            println!("No entries found.");
            return Ok(());
        }
        
        println!("\nExisting entries:");
        for (i, entry) in non_empty_entries.iter().enumerate() {
            if let Some(colon_pos) = entry.find(':') {
                let entry_id = &entry[..colon_pos];
                println!("{}: {}", i + 1, entry_id);
            } else {
                println!("{}: (entry without ID)", i + 1);
            }
        }
        
        print!("\nEnter the number of the entry to decrypt (or 0 to cancel): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice: usize = input.trim().parse().unwrap_or(0);
        
        if choice == 0 || choice > non_empty_entries.len() {
            return Ok(());
        }
        
        let selected_entry = non_empty_entries[choice - 1];
        if let Some(colon_pos) = selected_entry.find(':') {
            let entry_id = &selected_entry[..colon_pos];
            let encrypted_hex = &selected_entry[colon_pos + 1..];
            
            match hex::decode(encrypted_hex) {
                Ok(encrypted_bytes) => {
                    match decrypt_data(device, credential_id, &encrypted_bytes) {
                        Ok(decrypted_str) => {
                            println!("Decrypted data: {}", decrypted_str);
                        }
                        Err(_) => {
                            println!("Decryption error for '{}'", entry_id);
                        }
                    }
                }
                Err(_) => {
                    println!("Corrupted data in '{}'", entry_id);
                }
            }
        } else {
            // Handle old format without ID
            match hex::decode(selected_entry) {
                Ok(encrypted_bytes) => {
                    match decrypt_data(device, credential_id, &encrypted_bytes) {
                        Ok(decrypted_str) => {
                            println!("Entry {}: \"{}\"", choice, decrypted_str);
                        }
                        Err(_) => {
                            println!("Decryption error for entry {}", choice);
                        }
                    }
                }
                Err(_) => {
                    println!("Corrupted data in entry {}", choice);
                }
            }
        }
    } else {
        println!("Invalid data format in largeBlob.");
    }
    Ok(())
}

pub fn delete_single_entry(device: &mut FidoKeyHid) -> Result<()> {
    let current_blob = match device.get_large_blob() {
        Ok(response) => {
            if response.large_blob_array.is_empty() {
                println!("The largeBlob is empty.");
                return Ok(());
            }
            response.large_blob_array
        },
        Err(e) => return Err(anyhow!("Error reading blob: {}", e)),
    };

    let blob_content = String::from_utf8(current_blob)
        .context("Invalid data in largeBlob")?;
    
    if blob_content == hex::encode("EMPTY") {
        println!("The largeBlob is empty.");
        return Ok(());
    }
    
    let entries: Vec<&str> = blob_content.split('|').collect();
    let non_empty_entries: Vec<&str> = entries.iter().filter(|e| !e.is_empty()).cloned().collect();
    
    if non_empty_entries.is_empty() {
        println!("No entries to delete.");
        return Ok(());
    }
    
    println!("\nExisting entries:");
    for (i, entry) in non_empty_entries.iter().enumerate() {
        if let Some(colon_pos) = entry.find(':') {
            let entry_id = &entry[..colon_pos];
            println!("{}: {}", i + 1, entry_id);
        } else {
            println!("{}: (entry without ID)", i + 1);
        }
    }
    
    print!("Enter the number of the entry to delete (or 0 to cancel): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let key_index: usize = input.trim().parse().unwrap_or(0);
    
    if key_index == 0 || key_index > non_empty_entries.len() {
        println!("Operation cancelled.");
        return Ok(());
    }
    
    let mut updated_entries = non_empty_entries;
    updated_entries.remove(key_index - 1);

    // Request PIN only when needed for the write operation
    let mut pin = get_pin_from_user()?;
    
    if updated_entries.is_empty() {
        let empty_placeholder = hex::encode("EMPTY").into_bytes();
        
        match device.write_large_blob(Some(&pin), empty_placeholder) {
            Ok(_) => {
                println!("✓ LargeBlob cleared!");
            },
            Err(e) => {
                // Clear PIN from memory before returning error
                unsafe {
                    let bytes = pin.as_bytes_mut();
                    bytes.fill(0);
                }
                return Err(anyhow!("Failed to clear: {}", e));
            }
        }
    } else {
        let data = updated_entries.join("|").into_bytes();
        
        match device.write_large_blob(Some(&pin), data) {
            Ok(_) => {
                println!("✓ Entry deleted successfully!");
            },
            Err(e) => {
                // Clear PIN from memory before returning error
                unsafe {
                    let bytes = pin.as_bytes_mut();
                    bytes.fill(0);
                }
                return Err(anyhow!("Failed to update: {}", e));
            }
        }
    }
    
    unsafe {
        let bytes = pin.as_bytes_mut();
        bytes.fill(0);
    }
    
    Ok(())
}
