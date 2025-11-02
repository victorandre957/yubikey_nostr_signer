mod auth;
mod blob_operations;
mod credential;
mod device;
mod encryption;
mod yubikey_bunker;
mod yubikey_helper;

use anyhow::{Context, Result, anyhow};
use std::io::{self, Write};

use blob_operations::{delete_single_entry, read_blob, write_blob};
use credential::get_credential_id;
use device::{find_fido_device, is_supported};
use yubikey_bunker::YubikeyNostrBunker;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 YubiKey Nostr Manager\n");

    loop {
        println!("\n📋 Main Menu:");
        println!("1. 🔑 Manage Keys");
        println!("2. 🚀 Start NIP-46 Bunker");
        println!("3. 🚪 Exit");
        print!("\nOption (1-3): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                if let Err(e) = manage_keys().await {
                    eprintln!("❌ Error: {}", e);
                }
            }
            "2" => {
                if let Err(e) = start_bunker().await {
                    eprintln!("❌ Error starting bunker: {}", e);
                }
            }
            "3" => {
                println!("👋 Exiting...");
                break;
            }
            _ => {
                println!("❌ Invalid option.");
            }
        }
    }

    Ok(())
}

async fn manage_keys() -> Result<()> {
    let mut device = find_fido_device().context("No FIDO2 device found.")?;
    println!("✅ FIDO2 device connected!");

    if !is_supported(&device)? {
        return Err(anyhow!("This device does not support largeBlob."));
    }

    let credential_id =
        get_credential_id(&mut device).context("Failed to configure credential.")?;

    loop {
        println!("\n🔑 Key Management:");
        println!("1. 💾 Store key");
        println!("2. 👀 Read key");
        println!("3. 🗑️  Delete key");
        println!("4. ⬅️  Back");
        print!("\nOption (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                print!("\n📝 Enter private key (hex): ");
                io::stdout().flush()?;
                let mut data_input = String::new();
                io::stdin().read_line(&mut data_input)?;
                let data_to_write = data_input.trim();

                if let Err(e) = write_blob(&mut device, &credential_id, data_to_write) {
                    println!("❌ Error: {}", e);
                }
            }
            "2" => {
                if let Err(e) = read_blob(&mut device, &credential_id) {
                    println!("❌ Error: {}", e);
                }
            }
            "3" => {
                if let Err(e) = delete_single_entry(&mut device) {
                    println!("❌ Error: {}", e);
                }
            }
            "4" => {
                break;
            }
            _ => {
                println!("❌ Invalid option.");
            }
        }
    }

    Ok(())
}

async fn start_bunker() -> Result<()> {
    println!("\n🚀 Starting NIP-46 Bunker...\n");

    dotenvy::dotenv().context(".env file not found")?;

    let relays_str = std::env::var("NOSTR_RELAYS").context("NOSTR_RELAYS not defined in .env")?;

    let relays: Vec<&str> = relays_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if relays.is_empty() {
        anyhow::bail!("No relays configured");
    }

    println!("📡 Relays:");
    for relay in &relays {
        println!("   - {}", relay);
    }
    println!();

    let secret = Some("yubikey-secure-token".to_string());

    let bunker = YubikeyNostrBunker::new(relays, secret).context("Failed to initialize bunker")?;

    println!("💡 Share the URI above with Nostr apps");
    println!("🔒 Key loaded on-demand for each operation");
    println!("   Press Ctrl+C to stop\n");

    bunker.serve().await.context("Error running bunker")?;

    Ok(())
}
