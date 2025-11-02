use anyhow::Result;
use nostr::prelude::*;
use nostr_connect::prelude::*;
use std::io::{self, Write};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Test Client - Nostr Bunker\n");
    println!("This client will connect to the bunker and test NIP-46 operations\n");
    println!("============================================================\n");

    print!("Paste the bunker:// URI: ");
    io::stdout().flush()?;
    let mut bunker_uri = String::new();
    io::stdin().read_line(&mut bunker_uri)?;
    let bunker_uri = bunker_uri.trim();

    println!("\n📡 Connecting to bunker...\n");

    let uri = NostrConnectURI::parse(bunker_uri)?;

    println!("✅ URI parsed:");
    match &uri {
        NostrConnectURI::Bunker {
            remote_signer_public_key,
            relays,
            secret,
        } => {
            println!(
                "   Bunker pubkey: {}",
                remote_signer_public_key.to_bech32()?
            );
            println!("   Relays: {}", relays.len());
            for relay in relays {
                println!("      - {}", relay);
            }
            if secret.is_some() {
                println!("   Secret: Configured");
            }
        }
        _ => println!("   Client-type URI"),
    }
    println!();

    println!("🔐 Creating client keys...");
    let app_keys = Keys::generate();
    println!("   Client pubkey: {}\n", app_keys.public_key().to_bech32()?);

    println!("🔗 Establishing NIP-46 connection...");
    let signer = NostrConnect::new(uri, app_keys, Duration::from_secs(120), None)?;

    println!("⏳ Waiting for bunker approval...\n");
    println!("   👉 Go to the bunker terminal and approve the connection!\n");

    println!("📋 Requesting public key...");
    let bunker_pubkey = signer.get_public_key().await?;
    println!("✅ Public key received: {}\n", bunker_pubkey.to_bech32()?);

    loop {
        println!("\n============================================================");
        println!("📋 Test Menu:");
        println!("1. ✍️  Sign event (kind 1 - note)");
        println!("2. 🔐 NIP-04 Encrypt (legacy DM)");
        println!("3. 🔓 NIP-04 Decrypt");
        println!("4. 🔐 NIP-44 Encrypt (modern DM)");
        println!("5. 🔓 NIP-44 Decrypt");
        println!("6. 🚪 Exit");
        println!("============================================================");

        print!("\nChoose an option (1-6): ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => test_sign_event(&signer).await?,
            "2" => test_nip04_encrypt(&signer, &bunker_pubkey).await?,
            "3" => test_nip04_decrypt(&signer).await?,
            "4" => test_nip44_encrypt(&signer, &bunker_pubkey).await?,
            "5" => test_nip44_decrypt(&signer).await?,
            "6" => {
                println!("\n👋 Closing client...");
                break;
            }
            _ => println!("❌ Invalid option!"),
        }
    }

    Ok(())
}

async fn test_sign_event(signer: &NostrConnect) -> Result<()> {
    println!("\n--- Test: Sign Event ---");

    print!("Enter note content: ");
    io::stdout().flush()?;
    let mut content = String::new();
    io::stdin().read_line(&mut content)?;
    let content = content.trim();

    println!("📝 Creating and signing event...");
    let event = EventBuilder::text_note(content).sign(signer).await?;

    println!("✅ Event signed!");
    println!("   ID: {}", event.id);
    println!("   Pubkey: {}", event.pubkey.to_bech32()?);
    println!("   Content: {}", event.content);
    println!("   Signature: {}...", &event.sig.to_string()[..20]);

    print!("\nPublish to relay? (y/n): ");
    io::stdout().flush()?;
    let mut publish = String::new();
    io::stdin().read_line(&mut publish)?;

    if publish.trim().to_lowercase() == "y" || publish.trim().to_lowercase() == "yes" {
        let relay_url =
            std::env::var("RELAY_URL").unwrap_or_else(|_| "ws://relay:8080".to_string());

        println!("📡 Connecting to relay {}...", relay_url);

        use nostr_relay_pool::prelude::*;
        let pool = RelayPool::default();
        pool.add_relay(&relay_url, RelayOptions::default()).await?;
        pool.connect().await;

        println!("📤 Publishing event...");
        pool.send_event(&event).await?;

        println!("✅ Event published successfully to relay!");
        println!("   You can read it using: nak req -k 1 --limit 5 ws://relay:8080");
    }

    Ok(())
}

async fn test_nip04_encrypt(signer: &NostrConnect, receiver: &PublicKey) -> Result<()> {
    println!("\n--- Test: NIP-04 Encrypt ---");

    print!("Enter message to encrypt: ");
    io::stdout().flush()?;
    let mut plaintext = String::new();
    io::stdin().read_line(&mut plaintext)?;
    let plaintext = plaintext.trim();

    println!("🔐 Encrypting with NIP-04...");
    let encrypted = signer.nip04_encrypt(receiver, plaintext).await?;

    println!("✅ Message encrypted!");
    println!("   Ciphertext: {}", encrypted);

    Ok(())
}

async fn test_nip04_decrypt(signer: &NostrConnect) -> Result<()> {
    println!("\n--- Test: NIP-04 Decrypt ---");

    print!("Paste encrypted text: ");
    io::stdout().flush()?;
    let mut ciphertext = String::new();
    io::stdin().read_line(&mut ciphertext)?;
    let ciphertext = ciphertext.trim();

    print!("Paste sender pubkey (npub or hex): ");
    io::stdout().flush()?;
    let mut sender_str = String::new();
    io::stdin().read_line(&mut sender_str)?;
    let sender = PublicKey::parse(sender_str.trim())?;

    println!("🔓 Decrypting with NIP-04...");
    let decrypted = signer.nip04_decrypt(&sender, ciphertext).await?;

    println!("✅ Message decrypted!");
    println!("   Plaintext: {}", decrypted);

    Ok(())
}

async fn test_nip44_encrypt(signer: &NostrConnect, receiver: &PublicKey) -> Result<()> {
    println!("\n--- Test: NIP-44 Encrypt ---");

    print!("Enter message to encrypt: ");
    io::stdout().flush()?;
    let mut plaintext = String::new();
    io::stdin().read_line(&mut plaintext)?;
    let plaintext = plaintext.trim();

    println!("🔐 Encrypting with NIP-44...");
    let encrypted = signer.nip44_encrypt(receiver, plaintext).await?;

    println!("✅ Message encrypted!");
    println!("   Ciphertext: {}", encrypted);

    Ok(())
}

async fn test_nip44_decrypt(signer: &NostrConnect) -> Result<()> {
    println!("\n--- Test: NIP-44 Decrypt ---");

    print!("Paste encrypted text: ");
    io::stdout().flush()?;
    let mut ciphertext = String::new();
    io::stdin().read_line(&mut ciphertext)?;
    let ciphertext = ciphertext.trim();

    print!("Paste sender pubkey (npub or hex): ");
    io::stdout().flush()?;
    let mut sender_str = String::new();
    io::stdin().read_line(&mut sender_str)?;
    let sender = PublicKey::parse(sender_str.trim())?;

    println!("🔓 Decrypting with NIP-44...");
    let decrypted = signer.nip44_decrypt(&sender, ciphertext).await?;

    println!("✅ Message decrypted!");
    println!("   Plaintext: {}", decrypted);

    Ok(())
}
