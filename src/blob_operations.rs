use anyhow::{anyhow, Context, Result};
use ctap_hid_fido2::fidokey::FidoKeyHid;
use std::io::{self, Write};
use crate::auth::get_pin_from_user;
use crate::encryption::{encrypt_data, decrypt_data};
use base64::{Engine as _, engine::general_purpose};

// =============================================================================
// USER INPUT FUNCTIONS
// =============================================================================

/// Gets an entry ID from user input
fn get_entry_id() -> Result<String> {
    print!("Enter an ID for this entry: ");
    io::stdout().flush()?;
    let mut id_input = String::new();
    io::stdin().read_line(&mut id_input)?;
    let entry_id = id_input.trim().to_string();
    
    if entry_id.is_empty() {
        return Err(anyhow!("ID cannot be empty"));
    }
    
    Ok(entry_id)
}

/// Gets user choice for entry selection
fn get_user_choice(prompt: &str) -> Result<usize> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().parse().unwrap_or(0))
}

// =============================================================================
// BLOB DATA PARSING AND VALIDATION FUNCTIONS
// =============================================================================

/// Parses blob content and returns non-empty entries
fn parse_blob_entries(blob_content: &str) -> Vec<String> {
    if blob_content == general_purpose::STANDARD.encode("EMPTY") {
        return Vec::new();
    }
    
    blob_content.split('|')
        .filter(|e| !e.is_empty())
        .map(|e| e.to_string())
        .collect()
}

/// Checks if blob is empty or contains the empty placeholder
fn is_blob_empty(blob_data: &[u8]) -> bool {
    if blob_data.is_empty() {
        return true;
    }
    
    if let Ok(content) = String::from_utf8(blob_data.to_vec()) {
        content == general_purpose::STANDARD.encode("EMPTY") || content == hex::encode("EMPTY")
    } else {
        false
    }
}

/// Gets existing blob content as string
fn get_blob_content(device: &mut FidoKeyHid) -> Result<Option<String>> {
    let result = device.get_large_blob()
        .context("Failed to read from largeBlob")?;
    
    if is_blob_empty(&result.large_blob_array) {
        return Ok(None);
    }
    
    String::from_utf8(result.large_blob_array)
        .context("Invalid data in largeBlob")
        .map(Some)
}

// =============================================================================
// ENTRY DISPLAY AND SELECTION FUNCTIONS
// =============================================================================

/// Displays entries with their indices
fn display_entries(entries: &[String], title: &str) {
    println!("\n{}:", title);
    for (i, entry) in entries.iter().enumerate() {
        if let Some(colon_pos) = entry.find(':') {
            let entry_id = &entry[..colon_pos];
            println!("{}: {}", i + 1, entry_id);
        } else {
            println!("{}: (entry without ID)", i + 1);
        }
    }
}

/// Selects an entry based on user choice
fn select_entry(entries: &[String], choice: usize) -> Option<&String> {
    if choice > 0 && choice <= entries.len() {
        Some(&entries[choice - 1])
    } else {
        None
    }
}

// =============================================================================
// SPACE MANAGEMENT FUNCTIONS
// =============================================================================

/// Handles space management when adding a new entry
fn handle_space_management(existing_entries: &[String], new_entry: &str) -> Result<Vec<String>> {
    const MAX_SIZE: usize = 1024;
    let current_size = existing_entries.join("|").len();
    let needed_space = current_size + new_entry.len() + 1; // +1 for separator
    
    if needed_space <= MAX_SIZE {
        return Ok(existing_entries.to_vec());
    }
    
    println!("Insufficient space ({}/{} bytes).", needed_space, MAX_SIZE);
    display_entries(existing_entries, "Existing entries");
    
    let choice = get_user_choice("Enter the entry number to remove (or 0 to cancel): ")?;
    
    if choice == 0 {
        return Err(anyhow!("Operation cancelled"));
    }
    
    if let Some(_) = select_entry(existing_entries, choice) {
        let mut updated_entries = existing_entries.to_vec();
        updated_entries.remove(choice - 1);
        println!("Entry {} removed.", choice);
        Ok(updated_entries)
    } else {
        Err(anyhow!("Invalid choice"))
    }
}

// =============================================================================
// DEVICE WRITE OPERATIONS
// =============================================================================

/// Securely writes data to the device with PIN handling
fn write_to_device(device: &mut FidoKeyHid, data: Vec<u8>) -> Result<()> {
    let mut pin = get_pin_from_user()?;
    
    let result = device.write_large_blob(Some(pin.as_str()), data);
    
    // Clear PIN from memory
    unsafe {
        let bytes = pin.as_bytes_mut();
        bytes.fill(0);
    }
    
    result.map(|_| ()).context("Error writing to largeBlob")
}

/// Builds final data for writing to blob
fn build_final_data(existing_entries: Vec<String>, new_entry: String) -> Vec<u8> {
    if existing_entries.is_empty() {
        new_entry.into_bytes()
    } else {
        let mut all_entries = existing_entries;
        all_entries.push(new_entry);
        all_entries.join("|").into_bytes()
    }
}

// =============================================================================
// ENTRY DECRYPTION FUNCTIONS
// =============================================================================

/// Decrypts and displays a selected entry
fn decrypt_and_display_entry(
    device: &mut FidoKeyHid,
    credential_id: &[u8],
    entry: &str,
    entry_number: usize,
) -> Result<()> {
    if let Some(colon_pos) = entry.find(':') {
        let entry_id = &entry[..colon_pos];
        let encrypted_base64 = &entry[colon_pos + 1..];
        
        match general_purpose::STANDARD.decode(encrypted_base64) {
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
        // Handle old format without ID - try both base64 and hex for backward compatibility
        if let Ok(encrypted_bytes) = general_purpose::STANDARD.decode(entry) {
            match decrypt_data(device, credential_id, &encrypted_bytes) {
                Ok(decrypted_str) => {
                    println!("Entry {}: \"{}\"", entry_number, decrypted_str);
                }
                Err(_) => {
                    println!("Decryption error for entry {}", entry_number);
                }
            }
        } else if let Ok(encrypted_bytes) = hex::decode(entry) {
            // Fallback to hex for backward compatibility
            match decrypt_data(device, credential_id, &encrypted_bytes) {
                Ok(decrypted_str) => {
                    println!("Entry {}: \"{}\"", entry_number, decrypted_str);
                }
                Err(_) => {
                    println!("Decryption error for entry {}", entry_number);
                }
            }
        } else {
            println!("Corrupted data in entry {}", entry_number);
        }
    }
    Ok(())
}

// =============================================================================
// PUBLIC API FUNCTIONS
// =============================================================================

pub fn write_blob(device: &mut FidoKeyHid, credential_id: &[u8], data: &str) -> Result<()> {
    let entry_id = get_entry_id().context("Failed to get entry ID")?;
    
    println!("Encrypting data...");
    let encrypted_data = encrypt_data(device, credential_id, data)
        .context("Failed to encrypt data")?;
    
    // Format: "ID:encrypted_base64" (instead of hex)
    let entry_with_id = format!("{}:{}", entry_id, general_purpose::STANDARD.encode(&encrypted_data));
    
    // Get existing blob content
    let existing_entries = match get_blob_content(device)? {
        Some(content) => parse_blob_entries(&content),
        None => Vec::new(),
    };
    
    // Handle space management if needed
    let final_entries = handle_space_management(&existing_entries, &entry_with_id)?;
    
    // Build final data and write to device
    let final_data = build_final_data(final_entries, entry_with_id);
    write_to_device(device, final_data)?;
    
    println!("✓ Data encrypted and stored successfully!");
    Ok(())
}

pub fn read_blob(device: &mut FidoKeyHid, credential_id: &[u8]) -> Result<()> {
    println!("Reading data...");
    
    let blob_content = match get_blob_content(device)? {
        Some(content) => content,
        None => {
            println!("The largeBlob is empty.");
            return Ok(());
        }
    };
    
    let entries = parse_blob_entries(&blob_content);
    
    if entries.is_empty() {
        println!("No entries found.");
        return Ok(());
    }
    
    display_entries(&entries, "Existing entries");
    
    let choice = get_user_choice("\nEnter the number of the entry to decrypt (or 0 to cancel): ")?;
    
    if choice == 0 || choice > entries.len() {
        return Ok(());
    }
    
    let selected_entry = &entries[choice - 1];
    decrypt_and_display_entry(device, credential_id, selected_entry, choice)?;
    
    Ok(())
}

pub fn delete_single_entry(device: &mut FidoKeyHid) -> Result<()> {
    let blob_content = match get_blob_content(device)? {
        Some(content) => content,
        None => {
            println!("The largeBlob is empty.");
            return Ok(());
        }
    };
    
    let entries = parse_blob_entries(&blob_content);
    
    if entries.is_empty() {
        println!("No entries to delete.");
        return Ok(());
    }
    
    display_entries(&entries, "Existing entries");
    
    let choice = get_user_choice("Enter the number of the entry to delete (or 0 to cancel): ")?;
    
    if choice == 0 {
        println!("Operation cancelled.");
        return Ok(());
    }
    
    if let Some(_) = select_entry(&entries, choice) {
        let mut updated_entries = entries;
        updated_entries.remove(choice - 1);
        
        let final_data = if updated_entries.is_empty() {
            general_purpose::STANDARD.encode("EMPTY").into_bytes()
        } else {
            updated_entries.join("|").into_bytes()
        };
        
        write_to_device(device, final_data)?;
        
        if updated_entries.is_empty() {
            println!("✓ LargeBlob cleared!");
        } else {
            println!("✓ Entry deleted successfully!");
        }
    } else {
        println!("Invalid choice.");
    }
    
    Ok(())
}
