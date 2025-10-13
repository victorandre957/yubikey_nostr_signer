use anyhow::{Context, Result};
use nostr::prelude::*;
use yubikey_fido2_teste::NostrBunker;

/// Exemplo de uso do Nostr Bunker (NIP-46)
/// 
/// Este binário inicia um bunker que escuta requisições de clientes
/// e permite assinar eventos de forma segura.
#[tokio::main]
async fn main() -> Result<()> {
    // Inicializa o logger
    tracing_subscriber::fmt::init();

    println!("🚀 Iniciando Nostr Bunker (NIP-46)...\n");

    // Para testes, gera chaves aleatórias
    // Em produção, você leria essas chaves da YubiKey
    let signer_key = Keys::generate();
    let user_key = Keys::generate();

    println!("📌 Chaves geradas:");
    println!("   Signer pubkey: {}", signer_key.public_key().to_bech32()?);
    println!("   User pubkey: {}", user_key.public_key().to_bech32()?);
    println!();

    // Lista de relays para usar
    let relays = vec![
        "wss://relay.damus.io",
        "wss://nos.lol",
        "wss://relay.nostr.band",
    ];

    // Cria o bunker
    let bunker = NostrBunker::new(
        signer_key,
        user_key,
        relays,
        Some("secret-token-123".to_string()), // Segredo opcional para autorização automática
    )?;

    // Exibe o URI do bunker
    println!("🔗 Compartilhe este URI com clientes:");
    println!("   {}\n", bunker.bunker_uri());
    println!("💡 Dica: Use este URI em aplicativos como Amethyst, Damus, etc.\n");

    // Inicia o servidor bunker
    bunker.serve().await.context("Erro ao executar o bunker")?;

    Ok(())
}
