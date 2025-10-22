use anyhow::Result;
use nostr::prelude::*;
use nostr_connect::prelude::*;
use std::io::{self, Write};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Cliente de Teste - Nostr Bunker\n");
    println!("Este cliente irá conectar ao bunker e testar as operações NIP-46\n");
    println!("============================================================\n");

    print!("Cole o bunker:// URI: ");
    io::stdout().flush()?;
    let mut bunker_uri = String::new();
    io::stdin().read_line(&mut bunker_uri)?;
    let bunker_uri = bunker_uri.trim();

    println!("\n📡 Conectando ao bunker...\n");

    let uri = NostrConnectURI::parse(bunker_uri)?;

    println!("✅ URI parseado:");
    match &uri {
        NostrConnectURI::Bunker {
            remote_signer_public_key,
            relays,
            secret,
        } => {
            println!(
                "   Pubkey do bunker: {}",
                remote_signer_public_key.to_bech32()?
            );
            println!("   Relays: {}", relays.len());
            for relay in relays {
                println!("      - {}", relay);
            }
            if secret.is_some() {
                println!("   Secret: Configurado");
            }
        }
        _ => println!("   URI tipo cliente"),
    }
    println!();

    println!("🔐 Criando chaves do cliente...");
    let app_keys = Keys::generate();
    println!("   Client pubkey: {}\n", app_keys.public_key().to_bech32()?);

    println!("🔗 Estabelecendo conexão NIP-46...");
    let signer = NostrConnect::new(uri, app_keys, Duration::from_secs(120), None)?;

    println!("⏳ Aguardando aprovação no bunker...\n");
    println!("   👉 Vá até o terminal do bunker e aprove a conexão!\n");

    println!("📋 Solicitando chave pública...");
    let bunker_pubkey = signer.get_public_key().await?;
    println!(
        "✅ Chave pública recebida: {}\n",
        bunker_pubkey.to_bech32()?
    );

    loop {
        println!("\n============================================================");
        println!("📋 Menu de Testes:");
        println!("1. ✍️  Assinar evento (kind 1 - nota)");
        println!("2. 🔐 NIP-04 Encrypt (DM legado)");
        println!("3. 🔓 NIP-04 Decrypt");
        println!("4. 🔐 NIP-44 Encrypt (DM moderno)");
        println!("5. 🔓 NIP-44 Decrypt");
        println!("6. 🚪 Sair");
        println!("============================================================");

        print!("\nEscolha uma opção (1-6): ");
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
                println!("\n👋 Encerrando cliente...");
                break;
            }
            _ => println!("❌ Opção inválida!"),
        }
    }

    Ok(())
}

async fn test_sign_event(signer: &NostrConnect) -> Result<()> {
    println!("\n--- Teste: Assinar Evento ---");

    print!("Digite o conteúdo da nota: ");
    io::stdout().flush()?;
    let mut content = String::new();
    io::stdin().read_line(&mut content)?;
    let content = content.trim();

    println!("📝 Criando e assinando evento...");
    let event = EventBuilder::text_note(content).sign(signer).await?;

    println!("✅ Evento assinado!");
    println!("   ID: {}", event.id);
    println!("   Pubkey: {}", event.pubkey.to_bech32()?);
    println!("   Content: {}", event.content);
    println!("   Signature: {}...", &event.sig.to_string()[..20]);

    print!("\nPublicar no relay? (s/n): ");
    io::stdout().flush()?;
    let mut publish = String::new();
    io::stdin().read_line(&mut publish)?;

    if publish.trim().to_lowercase() == "s" || publish.trim().to_lowercase() == "sim" {
        let relay_url =
            std::env::var("RELAY_URL").unwrap_or_else(|_| "ws://relay:8080".to_string());

        println!("📡 Conectando ao relay {}...", relay_url);

        use nostr_relay_pool::prelude::*;
        let pool = RelayPool::default();
        pool.add_relay(&relay_url, RelayOptions::default()).await?;
        pool.connect().await;

        println!("📤 Publicando evento...");
        pool.send_event(&event).await?;

        println!("✅ Evento publicado com sucesso no relay!");
        println!("   Você pode ler ele usando: nak req -k 1 --limit 5 ws://relay:8080");
    }

    Ok(())
}

async fn test_nip04_encrypt(signer: &NostrConnect, receiver: &PublicKey) -> Result<()> {
    println!("\n--- Teste: NIP-04 Encrypt ---");

    print!("Digite a mensagem para encriptar: ");
    io::stdout().flush()?;
    let mut plaintext = String::new();
    io::stdin().read_line(&mut plaintext)?;
    let plaintext = plaintext.trim();

    println!("🔐 Encriptando com NIP-04...");
    let encrypted = signer.nip04_encrypt(receiver, plaintext).await?;

    println!("✅ Mensagem encriptada!");
    println!("   Ciphertext: {}", encrypted);

    Ok(())
}

async fn test_nip04_decrypt(signer: &NostrConnect) -> Result<()> {
    println!("\n--- Teste: NIP-04 Decrypt ---");

    print!("Cole o texto encriptado: ");
    io::stdout().flush()?;
    let mut ciphertext = String::new();
    io::stdin().read_line(&mut ciphertext)?;
    let ciphertext = ciphertext.trim();

    print!("Cole a pubkey do remetente (npub ou hex): ");
    io::stdout().flush()?;
    let mut sender_str = String::new();
    io::stdin().read_line(&mut sender_str)?;
    let sender = PublicKey::parse(sender_str.trim())?;

    println!("🔓 Desencriptando com NIP-04...");
    let decrypted = signer.nip04_decrypt(&sender, ciphertext).await?;

    println!("✅ Mensagem desencriptada!");
    println!("   Plaintext: {}", decrypted);

    Ok(())
}

async fn test_nip44_encrypt(signer: &NostrConnect, receiver: &PublicKey) -> Result<()> {
    println!("\n--- Teste: NIP-44 Encrypt ---");

    print!("Digite a mensagem para encriptar: ");
    io::stdout().flush()?;
    let mut plaintext = String::new();
    io::stdin().read_line(&mut plaintext)?;
    let plaintext = plaintext.trim();

    println!("🔐 Encriptando com NIP-44...");
    let encrypted = signer.nip44_encrypt(receiver, plaintext).await?;

    println!("✅ Mensagem encriptada!");
    println!("   Ciphertext: {}", encrypted);

    Ok(())
}

async fn test_nip44_decrypt(signer: &NostrConnect) -> Result<()> {
    println!("\n--- Teste: NIP-44 Decrypt ---");

    print!("Cole o texto encriptado: ");
    io::stdout().flush()?;
    let mut ciphertext = String::new();
    io::stdin().read_line(&mut ciphertext)?;
    let ciphertext = ciphertext.trim();

    print!("Cole a pubkey do remetente (npub ou hex): ");
    io::stdout().flush()?;
    let mut sender_str = String::new();
    io::stdin().read_line(&mut sender_str)?;
    let sender = PublicKey::parse(sender_str.trim())?;

    println!("🔓 Desencriptando com NIP-44...");
    let decrypted = signer.nip44_decrypt(&sender, ciphertext).await?;

    println!("✅ Mensagem desencriptada!");
    println!("   Plaintext: {}", decrypted);

    Ok(())
}
