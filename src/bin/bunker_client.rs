use anyhow::{Context, Result};
use nostr::prelude::*;
use nostr_connect::prelude::*;
use std::time::Duration;

/// Cliente de exemplo que se conecta ao Nostr Bunker e solicita assinatura de eventos
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🔌 Cliente Nostr Connect (NIP-46)\n");

    // Cole aqui o URI do bunker que foi exibido quando você executou o servidor
    println!("Digite o URI do bunker (bunker://...):");
    let mut bunker_uri = String::new();
    std::io::stdin()
        .read_line(&mut bunker_uri)
        .context("Falha ao ler URI")?;
    let bunker_uri = bunker_uri.trim();

    // Chaves do app cliente
    let app_keys = Keys::generate();
    println!("📱 App pubkey: {}", app_keys.public_key().to_bech32()?);

    // Conecta ao bunker
    println!("\n🔄 Conectando ao bunker...");
    let uri = NostrConnectURI::parse(bunker_uri)?;
    let signer = NostrConnect::new(uri, app_keys, Duration::from_secs(120), None)?;

    println!("✅ Conectado!\n");

    // Obtém a chave pública do signer
    println!("🔑 Obtendo chave pública...");
    let pubkey = signer.get_public_key().await?;
    println!("   Pubkey: {}", pubkey.to_bech32()?);

    // Assina um evento de texto simples
    println!("\n📝 Solicitando assinatura de evento...");
    let unsigned = EventBuilder::text_note("Hello from Nostr Bunker! 🎉").build(pubkey);
    let event = signer.sign_event(unsigned).await?;
    
    println!("✅ Evento assinado com sucesso!");
    println!("   ID: {}", event.id);
    println!("   Content: {}", event.content);

    // Teste de encriptação NIP-04
    println!("\n🔐 Testando encriptação NIP-04...");
    let target_pubkey = Keys::generate().public_key();
    let plaintext = "Mensagem secreta usando NIP-04";
    let encrypted = signer.nip04_encrypt(&target_pubkey, plaintext).await?;
    println!("   Texto original: {}", plaintext);
    println!("   Encriptado: {}...", &encrypted[..encrypted.len().min(50)]);

    // Teste de encriptação NIP-44
    println!("\n🔐 Testando encriptação NIP-44...");
    let plaintext = "Mensagem secreta usando NIP-44";
    let encrypted = signer.nip44_encrypt(&target_pubkey, plaintext).await?;
    println!("   Texto original: {}", plaintext);
    println!("   Encriptado: {}...", &encrypted[..encrypted.len().min(50)]);

    println!("\n✨ Todos os testes concluídos com sucesso!");

    Ok(())
}
