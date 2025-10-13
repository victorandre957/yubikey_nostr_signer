mod auth;
mod blob_operations;
mod credential;
mod device;
mod encryption;
mod yubikey_bunker;
mod yubikey_keys;

use anyhow::{Context, Result, anyhow};
use std::io::{self, Write};

use blob_operations::{delete_single_entry, read_blob, write_blob};
use credential::get_credential_id;
use device::{find_fido_device, is_supported};
use yubikey_bunker::YubikeyNostrBunker;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔐 YubiKey Nostr Manager\n");
    println!("============================================================\n");

    loop {
        println!("\n📋 Menu Principal:");
        println!("1. 🔑 Gerenciar Chaves (Store/Read/Delete)");
        println!("2. 🚀 Iniciar Nostr Bunker (NIP-46)");
        println!("3. 🚪 Sair");
        print!("\nEscolha uma opção (1-3): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                if let Err(e) = manage_keys().await {
                    eprintln!("❌ Erro: {}", e);
                }
            }
            "2" => {
                if let Err(e) = start_bunker().await {
                    eprintln!("❌ Erro ao iniciar bunker: {}", e);
                }
            }
            "3" => {
                println!("👋 Saindo...");
                break;
            }
            _ => {
                println!("❌ Opção inválida.");
            }
        }
    }

    Ok(())
}

/// Gerencia chaves na YubiKey (menu de manipulação)
async fn manage_keys() -> Result<()> {
    let mut device = find_fido_device().context("Nenhum dispositivo FIDO2 encontrado.")?;
    println!("✅ Dispositivo FIDO2 conectado!");

    if !is_supported(&device)? {
        return Err(anyhow!("Este dispositivo não suporta largeBlob."));
    }

    let credential_id = get_credential_id(&mut device)
        .context("Falha ao configurar credencial.")?;

    loop {
        println!("\n🔑 Gerenciamento de Chaves:");
        println!("1. 💾 Armazenar chave (Store)");
        println!("2. 👀 Ler chave (Read)");
        println!("3. 🗑️  Deletar chave (Delete)");
        println!("4. ⬅️  Voltar ao menu principal");
        print!("\nEscolha uma opção (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                print!("\n📝 Digite os dados para criptografar (hex da chave privada Nostr): ");
                io::stdout().flush()?;
                let mut data_input = String::new();
                io::stdin().read_line(&mut data_input)?;
                let data_to_write = data_input.trim();

                if let Err(e) = write_blob(&mut device, &credential_id, data_to_write) {
                    println!("❌ Erro: {}", e);
                }
            }
            "2" => {
                if let Err(e) = read_blob(&mut device, &credential_id) {
                    println!("❌ Erro: {}", e);
                }
            }
            "3" => {
                if let Err(e) = delete_single_entry(&mut device) {
                    println!("❌ Erro: {}", e);
                }
            }
            "4" => {
                break;
            }
            _ => {
                println!("❌ Opção inválida.");
            }
        }
    }

    Ok(())
}

/// Inicia o Nostr Bunker com YubiKey
async fn start_bunker() -> Result<()> {
    println!("\n🚀 Iniciando Nostr Bunker com YubiKey...\n");
    println!("============================================================\n");

    // Carrega arquivo .env (obrigatório)
    dotenvy::dotenv()
        .context("Arquivo .env não encontrado. Crie um arquivo .env com NOSTR_RELAYS configurado.")?;

    // Lê relays do .env (obrigatório)
    let relays_str = std::env::var("NOSTR_RELAYS")
        .context("NOSTR_RELAYS não definido no .env. Adicione: NOSTR_RELAYS=wss://relay1.io,wss://relay2.io")?;

    // Separa os relays por vírgula
    let relays: Vec<&str> = relays_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if relays.is_empty() {
        anyhow::bail!("Nenhum relay configurado em NOSTR_RELAYS");
    }

    println!("📡 Relays configurados:");
    for relay in &relays {
        println!("   - {}", relay);
    }
    println!();

    // Segredo opcional para autorização automática
    let secret = Some("yubikey-secure-token".to_string());

    println!("============================================================\n");

    // Cria e inicia o bunker
    let bunker = YubikeyNostrBunker::new(relays, secret)
        .context("Falha ao inicializar bunker com YubiKey")?;

    println!("============================================================\n");
    println!("💡 Como usar:");
    println!("   1. Compartilhe o URI acima com aplicativos Nostr");
    println!("   2. Aprove as requisições quando aparecerem");
    println!("   3. A chave será lida da YubiKey para cada operação");
    println!("   4. Pressione Ctrl+C para encerrar");
    println!();
    println!("🔒 Segurança:");
    println!("   • Chave carregada SOB DEMANDA para cada assinatura");
    println!("   • PIN necessário para cada leitura");
    println!();
    println!("============================================================\n");

    // Inicia o servidor
    bunker.serve().await.context("Erro ao executar o bunker")?;

    Ok(())
}
