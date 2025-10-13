use anyhow::Result;
use dialoguer::Confirm;
use nostr::prelude::*;
use nostr_relay_pool::prelude::*;
use std::sync::Arc;

use crate::yubikey_keys::YubikeyKeyManager;

/// Bunker que usa YubiKey para armazenar a chave privada
/// A chave é carregada sob demanda para cada operação e imediatamente descartada
pub struct YubikeyNostrBunker {
    /// Chave temporária para comunicação NIP-46 (fica em memória)
    signer_key: Keys,
    /// Gerenciador de chaves da YubiKey
    yubikey_manager: Arc<YubikeyKeyManager>,
    /// Pool de relays
    pool: RelayPool,
    /// Relays configurados
    relays: Vec<String>,
    /// Segredo opcional para autorização
    secret: Option<String>,
}

impl YubikeyNostrBunker {
    /// Cria um novo bunker conectado à YubiKey
    pub fn new<I, S>(
        relays: I,
        secret: Option<String>,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Inicializa o gerenciador da YubiKey
        let yubikey_manager = Arc::new(YubikeyKeyManager::new()?);

        // Gera chave temporária para comunicação NIP-46
        // Esta chave fica em memória apenas para decrypt/encrypt das mensagens do protocolo
        let signer_key = Keys::generate();
        
        println!("🔐 Chave temporária NIP-46 gerada:");
        println!("   Pubkey: {}\n", signer_key.public_key().to_bech32()?);

        let relay_urls: Vec<String> = relays.into_iter()
            .map(|r| r.as_ref().to_string())
            .collect();

        let pool = RelayPool::default();

        Ok(Self {
            signer_key,
            yubikey_manager,
            pool,
            relays: relay_urls,
            secret,
        })
    }

    /// Retorna o URI bunker:// para compartilhar com clientes
    pub fn bunker_uri(&self) -> Result<NostrConnectURI> {
        let relay_urls: Result<Vec<RelayUrl>, _> = self.relays
            .iter()
            .map(|r| RelayUrl::parse(r))
            .collect();

        Ok(NostrConnectURI::Bunker {
            remote_signer_public_key: self.signer_key.public_key(),
            relays: relay_urls?,
            secret: self.secret.clone(),
        })
    }

    /// Inicia o bunker e processa requisições
    pub async fn serve(self) -> Result<()> {
        println!("🔑 Nostr Bunker (YubiKey) iniciado!");
        println!("📋 Bunker URI: {}\n", self.bunker_uri()?);
        println!("⏳ Aguardando requisições...\n");

        // Conecta aos relays
        for relay_url in &self.relays {
            self.pool.add_relay(relay_url, RelayOptions::default()).await?;
        }
        self.pool.connect().await;

        // Obtém a chave pública para o filtro
        let user_pubkey = self.yubikey_manager.get_public_key()?;

        // Subscreve a eventos NIP-46 direcionados ao signer
        let filter = Filter::new()
            .kind(Kind::NostrConnect)
            .pubkey(self.signer_key.public_key())
            .since(Timestamp::now());

        self.pool.subscribe(filter, SubscribeOptions::default()).await?;

        // Loop de eventos
        let mut notifications = self.pool.notifications();
        
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind == Kind::NostrConnect {
                    // Processa o evento
                    if let Err(e) = self.handle_request(&event, &user_pubkey).await {
                        eprintln!("❌ Erro ao processar requisição: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Processa uma requisição NIP-46
    async fn handle_request(&self, event: &Event, user_pubkey: &PublicKey) -> Result<()> {
        // Decripta a mensagem usando a chave do signer
        let decrypted = nip44::decrypt(
            self.signer_key.secret_key(),
            &event.pubkey,
            &event.content,
        )?;

        // Parseia a mensagem NIP-46
        let msg: NostrConnectMessage = NostrConnectMessage::from_json(decrypted)?;

        println!("📨 Requisição recebida de: {}", event.pubkey);

        // Extrai a requisição
        let (id, request) = match msg {
            NostrConnectMessage::Request { id, method, params } => {
                let req = NostrConnectRequest::from_message(method, params)?;
                (id, req)
            }
            _ => {
                println!("⚠️  Mensagem não é uma requisição, ignorando");
                return Ok(());
            }
        };

        // Solicita aprovação do usuário
        if !self.should_approve(&event.pubkey, &request) {
            println!("❌ Requisição negada pelo usuário\n");
            
            // Envia resposta de erro
            let response = NostrConnectResponse::with_error("Requisição negada pelo usuário");
            self.send_response(&event.pubkey, &id, response).await?;
            return Ok(());
        }

        // Processa a requisição baseado no tipo
        let response = match request {
            NostrConnectRequest::Connect { .. } => {
                println!("✅ Conexão aprovada\n");
                NostrConnectResponse::with_result(ResponseResult::Ack)
            }
            NostrConnectRequest::GetPublicKey => {
                println!("✅ Chave pública enviada\n");
                NostrConnectResponse::with_result(
                    ResponseResult::GetPublicKey(*user_pubkey)
                )
            }
            NostrConnectRequest::SignEvent(unsigned) => {
                println!("📝 Assinando evento com YubiKey...");
                
                // Carrega a chave da YubiKey, assina e descarta
                match self.yubikey_manager.with_key(|keys| {
                    unsigned.sign_with_keys(keys).map_err(|e| anyhow::anyhow!(e))
                }) {
                    Ok(signed_event) => {
                        println!("✅ Evento assinado com sucesso");
                        println!("   ID: {}\n", signed_event.id);
                        NostrConnectResponse::with_result(
                            ResponseResult::SignEvent(Box::new(signed_event))
                        )
                    }
                    Err(e) => {
                        eprintln!("❌ Erro ao assinar: {}\n", e);
                        NostrConnectResponse::with_error(format!("Erro ao assinar: {}", e))
                    }
                }
            }
            NostrConnectRequest::Nip04Encrypt { public_key, text } => {
                println!("🔐 Encriptando com NIP-04...");
                
                match self.yubikey_manager.with_key(|keys| {
                    nip04::encrypt(keys.secret_key(), &public_key, &text)
                        .map_err(|e| anyhow::anyhow!("Erro NIP-04: {}", e))
                }) {
                    Ok(ciphertext) => {
                        println!("✅ Encriptado com sucesso\n");
                        NostrConnectResponse::with_result(
                            ResponseResult::Nip04Encrypt { ciphertext }
                        )
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Erro: {}", e))
                }
            }
            NostrConnectRequest::Nip04Decrypt { public_key, ciphertext } => {
                println!("🔓 Decriptando com NIP-04...");
                
                match self.yubikey_manager.with_key(|keys| {
                    nip04::decrypt(keys.secret_key(), &public_key, &ciphertext)
                        .map_err(|e| anyhow::anyhow!("Erro NIP-04: {}", e))
                }) {
                    Ok(plaintext) => {
                        println!("✅ Decriptado com sucesso\n");
                        NostrConnectResponse::with_result(
                            ResponseResult::Nip04Decrypt { plaintext }
                        )
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Erro: {}", e))
                }
            }
            NostrConnectRequest::Nip44Encrypt { public_key, text } => {
                println!("🔐 Encriptando com NIP-44...");
                
                match self.yubikey_manager.with_key(|keys| {
                    nip44::encrypt(keys.secret_key(), &public_key, &text, nip44::Version::default())
                        .map_err(|e| anyhow::anyhow!("Erro NIP-44: {}", e))
                }) {
                    Ok(ciphertext) => {
                        println!("✅ Encriptado com sucesso\n");
                        NostrConnectResponse::with_result(
                            ResponseResult::Nip44Encrypt { ciphertext }
                        )
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Erro: {}", e))
                }
            }
            NostrConnectRequest::Nip44Decrypt { public_key, ciphertext } => {
                println!("🔓 Decriptando com NIP-44...");
                
                match self.yubikey_manager.with_key(|keys| {
                    nip44::decrypt(keys.secret_key(), &public_key, &ciphertext)
                        .map_err(|e| anyhow::anyhow!("Erro NIP-44: {}", e))
                }) {
                    Ok(plaintext) => {
                        println!("✅ Decriptado com sucesso\n");
                        NostrConnectResponse::with_result(
                            ResponseResult::Nip44Decrypt { plaintext }
                        )
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Erro: {}", e))
                }
            }
            NostrConnectRequest::Ping => {
                println!("🏓 Pong enviado\n");
                NostrConnectResponse::with_result(ResponseResult::Ack)
            }
        };

        // Envia a resposta
        self.send_response(&event.pubkey, &id, response).await?;

        Ok(())
    }

    /// Envia uma resposta para o cliente
    async fn send_response(
        &self,
        client_pubkey: &PublicKey,
        request_id: &str,
        response: NostrConnectResponse,
    ) -> Result<()> {
        let msg = NostrConnectMessage::response(request_id, response);

        // Encripta a resposta
        let encrypted = nip44::encrypt(
            self.signer_key.secret_key(),
            client_pubkey,
            msg.as_json(),
            nip44::Version::default(),
        )?;

        // Cria o evento
        let event = EventBuilder::new(Kind::NostrConnect, encrypted)
            .tag(Tag::public_key(*client_pubkey))
            .sign_with_keys(&self.signer_key)?;

        // Envia para os relays
        self.pool.send_event(&event).await?;

        println!("📤 Resposta enviada\n");

        Ok(())
    }

    /// Verifica se deve aprovar a requisição
    fn should_approve(&self, client_pubkey: &PublicKey, request: &NostrConnectRequest) -> bool {
        match request {
            NostrConnectRequest::Connect { public_key: req_pk, .. } => {
                println!("\n🔔 Nova solicitação de conexão!");
                println!("   De: {}", client_pubkey);
                println!("   App pubkey: {}", req_pk);
                
                Confirm::new()
                    .with_prompt("Aprovar conexão?")
                    .default(false)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::GetPublicKey => {
                println!("🔑 Solicitação para obter chave pública de {}", client_pubkey);
                true // Sempre permite
            }
            NostrConnectRequest::SignEvent(event) => {
                println!("\n📝 Solicitação para assinar evento:");
                println!("   De: {}", client_pubkey);
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
                println!("   De: {}", client_pubkey);
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
                println!("   De: {}", client_pubkey);
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
                println!("   De: {}", client_pubkey);
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
                println!("   De: {}", client_pubkey);
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
                println!("🏓 Ping recebido de {}", client_pubkey);
                true // Sempre responde a pings
            }
        }
    }
}
