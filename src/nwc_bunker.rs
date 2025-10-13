use anyhow::{Context, Result};
use dialoguer::Confirm;
use nostr::prelude::*;
use nostr_connect::prelude::*;

/// Implementação de um Nostr Bunker (NIP-46) que escuta requisições de clientes
/// e as processa usando chaves armazenadas de forma segura.
pub struct NostrBunker {
    signer: NostrConnectRemoteSigner,
}

impl NostrBunker {
    /// Cria um novo bunker com as chaves fornecidas e relays especificados.
    /// 
    /// # Argumentos
    /// * `signer_key` - Chave privada do signer (para assinar mensagens NIP-46)
    /// * `user_key` - Chave privada do usuário (para assinar eventos)
    /// * `relays` - Lista de relays para conectar
    /// * `secret` - Segredo opcional para autorização automática
    pub fn new<I, S>(
        signer_key: Keys,
        user_key: Keys,
        relays: I,
        secret: Option<String>,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let keys = NostrConnectKeys { signer: signer_key, user: user_key };
        
        let relay_urls: Vec<String> = relays.into_iter()
            .map(|r| r.as_ref().to_string())
            .collect();

        let signer = NostrConnectRemoteSigner::new(keys, relay_urls, secret, None)
            .context("Falha ao criar NostrConnectRemoteSigner")?;

        Ok(Self { signer })
    }

    /// Retorna o URI bunker:// para compartilhar com clientes.
    pub fn bunker_uri(&self) -> NostrConnectURI {
        self.signer.bunker_uri()
    }

    /// Inicia o bunker e começa a escutar requisições.
    /// Esta função bloqueia até receber um sinal de parada.
    pub async fn serve(self) -> Result<()> {
        println!("\n🔑 Nostr Bunker iniciado!");
        println!("📋 Bunker URI: {}\n", self.bunker_uri());
        println!("⏳ Aguardando requisições...\n");

        self.signer
            .serve(BunkerActions)
            .await
            .context("Erro ao executar o bunker")?;

        Ok(())
    }
}

/// Implementação das ações do bunker - controla como o bunker responde a requisições.
struct BunkerActions;

impl NostrConnectSignerActions for BunkerActions {
    /// Solicita aprovação do usuário para processar uma requisição.
    /// Este método é chamado para cada tipo de requisição recebida.
    fn approve(&self, public_key: &PublicKey, req: &NostrConnectRequest) -> bool {
        match req {
            NostrConnectRequest::Connect { public_key: req_pk, .. } => {
                println!("\n🔔 Nova solicitação de conexão!");
                println!("   De: {}", public_key);
                println!("   App pubkey: {}", req_pk);
                
                Confirm::new()
                    .with_prompt("Aprovar conexão?")
                    .default(false)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::GetPublicKey => {
                println!("\n🔑 Solicitação para obter chave pública de {}", public_key);
                true // Sempre permite
            }
            NostrConnectRequest::SignEvent(event) => {
                println!("\n📝 Solicitação para assinar evento:");
                println!("   De: {}", public_key);
                println!("   Kind: {}", event.kind);
                println!("   Content: {}", 
                    if event.content.len() > 100 {
                        format!("{}...", &event.content[..100])
                    } else {
                        event.content.clone()
                    }
                );
                
                Confirm::new()
                    .with_prompt("Assinar este evento?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip04Encrypt { public_key: target, text } => {
                println!("\n🔐 Solicitação para encriptar (NIP-04):");
                println!("   De: {}", public_key);
                println!("   Para: {}", target);
                println!("   Texto: {}", 
                    if text.len() > 50 {
                        format!("{}...", &text[..50])
                    } else {
                        text.clone()
                    }
                );
                
                Confirm::new()
                    .with_prompt("Encriptar?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip04Decrypt { public_key: from, ciphertext } => {
                println!("\n🔓 Solicitação para decriptar (NIP-04):");
                println!("   De: {}", public_key);
                println!("   From pubkey: {}", from);
                println!("   Ciphertext: {}...", 
                    &ciphertext[..ciphertext.len().min(50)]
                );
                
                Confirm::new()
                    .with_prompt("Decriptar?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip44Encrypt { public_key: target, text } => {
                println!("\n🔐 Solicitação para encriptar (NIP-44):");
                println!("   De: {}", public_key);
                println!("   Para: {}", target);
                println!("   Texto: {}", 
                    if text.len() > 50 {
                        format!("{}...", &text[..50])
                    } else {
                        text.clone()
                    }
                );
                
                Confirm::new()
                    .with_prompt("Encriptar?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip44Decrypt { public_key: from, ciphertext } => {
                println!("\n🔓 Solicitação para decriptar (NIP-44):");
                println!("   De: {}", public_key);
                println!("   From pubkey: {}", from);
                println!("   Ciphertext: {}...", 
                    &ciphertext[..ciphertext.len().min(50)]
                );
                
                Confirm::new()
                    .with_prompt("Decriptar?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Ping => {
                println!("\n🏓 Ping recebido de {}", public_key);
                true // Sempre responde a pings
            }
        }
    }
}
