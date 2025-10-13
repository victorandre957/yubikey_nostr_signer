use anyhow::{Context, Result};
use yubikey_fido2_teste::YubikeyNostrBunker;

/// Nostr Bunker usando YubiKey para armazenar chaves privadas
/// 
/// Este bunker:
/// - Mantém a chave privada na YubiKey
/// - Carrega a chave SOB DEMANDA para cada operação
/// - Limpa a chave da memória imediatamente após uso
/// - Usa chave temporária apenas para protocolo NIP-46
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 Nostr Bunker com YubiKey (NIP-46)\n");
    println!("{}", "=".repeat(60));
    println!();

    // Lista de relays
    let relays = vec![
        "wss://relay.damus.io",
        "wss://nos.lol",
        "wss://relay.nostr.band",
    ];

    println!("📡 Relays configurados:");
    for relay in &relays {
        println!("   • {}", relay);
    }
    println!();

    // Segredo opcional para autorização automática
    let secret = Some("yubikey-secure-token".to_string());

    println!("{}", "=".repeat(60));
    println!();

    // Cria e inicia o bunker
    // Durante a criação, o usuário será solicitado a:
    // 1. Conectar a YubiKey
    // 2. Escolher qual chave usar (se houver múltiplas)
    // 3. Inserir o PIN para ler a chave
    let bunker = YubikeyNostrBunker::new(relays, secret)
        .context("Falha ao inicializar bunker com YubiKey")?;

    println!("{}", "=".repeat(60));
    println!();
    println!("💡 Como usar:");
    println!("   1. Compartilhe o URI acima com aplicativos Nostr");
    println!("   2. Aprove as requisições quando aparecerem");
    println!("   3. A chave será lida da YubiKey para cada operação");
    println!("   4. Pressione Ctrl+C para encerrar");
    println!();
    println!("🔒 Segurança:");
    println!("   • Chave privada NUNCA sai da YubiKey permanentemente");
    println!("   • Carregada SOB DEMANDA para cada assinatura");
    println!("   • Limpa da memória IMEDIATAMENTE após uso");
    println!("   • PIN necessário para cada leitura");
    println!();
    println!("{}", "=".repeat(60));
    println!();

    // Inicia o servidor
    bunker.serve().await.context("Erro ao executar o bunker")?;

    Ok(())
}
