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
    let mut device = find_fido_device().context("Nenhum dispositivo FIDO2 encontrado.")?;
    println!("Dispositivo FIDO2 conectado!");

    if !is_supported(&device)? {
        return Err(anyhow!("Este dispositivo não suporta largeBlob."));
    }

    let credential_id = get_credential_id(&mut device)
        .context("Falha ao configurar credencial.")?;

    loop {
        println!("\nEscolha uma opção:");
        println!("1 - Escrever dados");
        println!("2 - Ler dados");
        println!("3 - Apagar entrada");
        println!("4 - Sair");
        print!("Digite sua escolha (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                print!("Digite os dados para escrever: ");
                io::stdout().flush()?;
                let mut data_input = String::new();
                io::stdin().read_line(&mut data_input)?;
                let data_to_write = data_input.trim();
                
                if let Err(e) = write_blob(&mut device, &credential_id, data_to_write) {
                    println!("Erro: {}", e);
                }
            }
            "2" => {
                if let Err(e) = read_blob(&mut device, &credential_id) {
                    println!("Erro: {}", e);
                }
            }
            "3" => {
                if let Err(e) = delete_single_entry(&mut device) {
                    println!("Erro: {}", e);
                }
            }
            "4" => {
                println!("Saindo...");
                break;
            }
            _ => {
                println!("Opção inválida.");
            }
        }
    }

    Ok(())
}