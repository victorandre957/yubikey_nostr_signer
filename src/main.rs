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

    loop {
        println!("\n📋 Menu Principal:");
        println!("1. 🔑 Gerenciar Chaves");
        println!("2. 🚀 Iniciar Bunker NIP-46");
        println!("3. 🚪 Sair");
        print!("\nOpção (1-3): ");
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

async fn manage_keys() -> Result<()> {
    let mut device = find_fido_device().context("Nenhum dispositivo FIDO2 encontrado.")?;
    println!("✅ Dispositivo FIDO2 conectado!");

    if !is_supported(&device)? {
        return Err(anyhow!("Este dispositivo não suporta largeBlob."));
    }

    let credential_id =
        get_credential_id(&mut device).context("Falha ao configurar credencial.")?;

    loop {
        println!("\n🔑 Gerenciamento de Chaves:");
        println!("1. 💾 Armazenar chave");
        println!("2. 👀 Ler chave");
        println!("3. 🗑️  Deletar chave");
        println!("4. ⬅️  Voltar");
        print!("\nOpção (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                print!("\n📝 Digite a chave privada (hex): ");
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

async fn start_bunker() -> Result<()> {
    println!("\n🚀 Iniciando Bunker NIP-46...\n");

    dotenvy::dotenv().context("Arquivo .env não encontrado")?;

    let relays_str = std::env::var("NOSTR_RELAYS").context("NOSTR_RELAYS não definido no .env")?;

    let relays: Vec<&str> = relays_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if relays.is_empty() {
        anyhow::bail!("Nenhum relay configurado");
    }

    println!("📡 Relays:");
    for relay in &relays {
        println!("   - {}", relay);
    }
    println!();

    let secret = Some("yubikey-secure-token".to_string());

    let bunker = YubikeyNostrBunker::new(relays, secret).context("Falha ao inicializar bunker")?;

    println!("💡 Compartilhe o URI acima com aplicativos Nostr");
    println!("🔒 Chave carregada sob demanda para cada operação");
    println!("   Pressione Ctrl+C para encerrar\n");

    bunker.serve().await.context("Erro ao executar bunker")?;

    Ok(())
}
