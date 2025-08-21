mod auth;
mod device;
mod credential;
mod blob_operations;

use anyhow::{anyhow, Context, Result};
use std::io::{self, Write};

use device::{find_fido_device, is_supported};
use credential::get_credential_id;
use blob_operations::{write_blob, read_blob, delete_single_entry};

fn main() -> Result<()> {
    let mut device = find_fido_device().context("No FIDO2 device found.")?;
    println!("FIDO2 device connected!");

    if !is_supported(&device)? {
        return Err(anyhow!("This device does not support largeBlob."));
    }

    let credential_id = get_credential_id(&mut device)
        .context("Failed to configure credential.")?;

    loop {
        println!("\nSelect an option:");
        println!("1. Store encrypted data");
        println!("2. Read encrypted data");
        println!("3. Delete encrypted data");
        println!("4. Exit");
        print!("Enter your choice (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                print!("Enter data to encrypt: ");
                io::stdout().flush()?;
                let mut data_input = String::new();
                io::stdin().read_line(&mut data_input)?;
                let data_to_write = data_input.trim();
                
                if let Err(e) = write_blob(&mut device, &credential_id, data_to_write) {
                    println!("Error: {}", e);
                }
            }
            "2" => {
                if let Err(e) = read_blob(&mut device, &credential_id) {
                    println!("Error: {}", e);
                }
            }
            "3" => {
                if let Err(e) = delete_single_entry(&mut device) {
                    println!("Error: {}", e);
                }
            }
            "4" => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Invalid option.");
            }
        }
    }

    Ok(())
}