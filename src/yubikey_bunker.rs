use anyhow::Result;
use dialoguer::Confirm;
use nostr::prelude::*;
use nostr_relay_pool::prelude::*;
use std::sync::Arc;

use crate::yubikey_helper::YubikeyKeyManager;

pub struct YubikeyNostrBunker {
    signer_key: Keys,
    yubikey_manager: Arc<YubikeyKeyManager>,
    pool: RelayPool,
    relays: Vec<String>,
    secret: Option<String>,
}

impl YubikeyNostrBunker {
    pub fn new<I, S>(relays: I, secret: Option<String>) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let yubikey_manager = Arc::new(YubikeyKeyManager::new()?);
        let signer_key = Keys::generate();

        println!("🔐 Temporary NIP-46 key generated:");
        println!("   Pubkey: {}\n", signer_key.public_key().to_bech32()?);

        let relay_urls: Vec<String> = relays.into_iter().map(|r| r.as_ref().to_string()).collect();

        Ok(Self {
            signer_key,
            yubikey_manager,
            pool: RelayPool::default(),
            relays: relay_urls,
            secret,
        })
    }

    pub fn bunker_uri(&self) -> Result<NostrConnectURI> {
        let relay_urls: Result<Vec<RelayUrl>, _> =
            self.relays.iter().map(|r| RelayUrl::parse(r)).collect();

        Ok(NostrConnectURI::Bunker {
            remote_signer_public_key: self.signer_key.public_key(),
            relays: relay_urls?,
            secret: self.secret.clone(),
        })
    }

    pub async fn serve(self) -> Result<()> {
        println!("🔑 Nostr Bunker (YubiKey) started!");
        println!("📋 Bunker URI: {}\n", self.bunker_uri()?);
        println!("⏳ Waiting for requests...\n");

        for relay_url in &self.relays {
            self.pool
                .add_relay(relay_url, RelayOptions::default())
                .await?;
        }
        self.pool.connect().await;

        let user_pubkey = self.yubikey_manager.get_public_key()?;

        let filter = Filter::new()
            .kind(Kind::NostrConnect)
            .pubkey(self.signer_key.public_key())
            .since(Timestamp::now());

        self.pool
            .subscribe(filter, SubscribeOptions::default())
            .await?;

        let mut notifications = self.pool.notifications();

        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification
                && event.kind == Kind::NostrConnect
                && let Err(e) = self.handle_request(&event, &user_pubkey).await
            {
                eprintln!("❌ Error processing request: {}", e);
            }
        }

        Ok(())
    }

    async fn handle_request(&self, event: &Event, user_pubkey: &PublicKey) -> Result<()> {
        let decrypted =
            nip44::decrypt(self.signer_key.secret_key(), &event.pubkey, &event.content)?;

        let msg: NostrConnectMessage = NostrConnectMessage::from_json(decrypted)?;

        println!("📨 Request received from: {}", event.pubkey);

        let (id, request) = match msg {
            NostrConnectMessage::Request { id, method, params } => {
                let req = NostrConnectRequest::from_message(method, params)?;
                (id, req)
            }
            _ => {
                println!("⚠️  Message is not a request, ignoring");
                return Ok(());
            }
        };

        if !self.should_approve(&event.pubkey, &request) {
            println!("❌ Request denied by user\n");

            let response = NostrConnectResponse::with_error("Request denied by user");
            self.send_response(&event.pubkey, &id, response).await?;
            return Ok(());
        }

        let response = match request {
            NostrConnectRequest::Connect { .. } => {
                println!("✅ Connection approved\n");
                NostrConnectResponse::with_result(ResponseResult::Ack)
            }
            NostrConnectRequest::GetPublicKey => {
                println!("✅ Public key sent\n");
                NostrConnectResponse::with_result(ResponseResult::GetPublicKey(*user_pubkey))
            }
            NostrConnectRequest::SignEvent(unsigned) => {
                println!("📝 Signing event with YubiKey...");

                match self.yubikey_manager.with_key(|keys| {
                    unsigned
                        .sign_with_keys(keys)
                        .map_err(|e| anyhow::anyhow!(e))
                }) {
                    Ok(signed_event) => {
                        println!("✅ Event signed successfully");
                        println!("   ID: {}\n", signed_event.id);
                        NostrConnectResponse::with_result(ResponseResult::SignEvent(Box::new(
                            signed_event,
                        )))
                    }
                    Err(e) => {
                        eprintln!("❌ Error signing: {}\n", e);
                        NostrConnectResponse::with_error(format!("Error signing: {}", e))
                    }
                }
            }
            NostrConnectRequest::Nip04Encrypt { public_key, text } => {
                println!("🔐 Encrypting with NIP-04...");

                match self.yubikey_manager.with_key(|keys| {
                    nip04::encrypt(keys.secret_key(), &public_key, &text)
                        .map_err(|e| anyhow::anyhow!("NIP-04 error: {}", e))
                }) {
                    Ok(ciphertext) => {
                        println!("✅ Encrypted successfully\n");
                        NostrConnectResponse::with_result(ResponseResult::Nip04Encrypt {
                            ciphertext,
                        })
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Error: {}", e)),
                }
            }
            NostrConnectRequest::Nip04Decrypt {
                public_key,
                ciphertext,
            } => {
                println!("🔓 Decrypting with NIP-04...");

                match self.yubikey_manager.with_key(|keys| {
                    nip04::decrypt(keys.secret_key(), &public_key, &ciphertext)
                        .map_err(|e| anyhow::anyhow!("NIP-04 error: {}", e))
                }) {
                    Ok(plaintext) => {
                        println!("✅ Decrypted successfully\n");
                        NostrConnectResponse::with_result(ResponseResult::Nip04Decrypt {
                            plaintext,
                        })
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Error: {}", e)),
                }
            }
            NostrConnectRequest::Nip44Encrypt { public_key, text } => {
                println!("🔐 Encrypting with NIP-44...");

                match self.yubikey_manager.with_key(|keys| {
                    nip44::encrypt(
                        keys.secret_key(),
                        &public_key,
                        &text,
                        nip44::Version::default(),
                    )
                    .map_err(|e| anyhow::anyhow!("NIP-44 error: {}", e))
                }) {
                    Ok(ciphertext) => {
                        println!("✅ Encrypted successfully\n");
                        NostrConnectResponse::with_result(ResponseResult::Nip44Encrypt {
                            ciphertext,
                        })
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Error: {}", e)),
                }
            }
            NostrConnectRequest::Nip44Decrypt {
                public_key,
                ciphertext,
            } => {
                println!("🔓 Decrypting with NIP-44...");

                match self.yubikey_manager.with_key(|keys| {
                    nip44::decrypt(keys.secret_key(), &public_key, &ciphertext)
                        .map_err(|e| anyhow::anyhow!("NIP-44 error: {}", e))
                }) {
                    Ok(plaintext) => {
                        println!("✅ Decrypted successfully\n");
                        NostrConnectResponse::with_result(ResponseResult::Nip44Decrypt {
                            plaintext,
                        })
                    }
                    Err(e) => NostrConnectResponse::with_error(format!("Error: {}", e)),
                }
            }
            NostrConnectRequest::Ping => {
                println!("🏓 Pong sent\n");
                NostrConnectResponse::with_result(ResponseResult::Ack)
            }
        };

        self.send_response(&event.pubkey, &id, response).await?;

        Ok(())
    }

    async fn send_response(
        &self,
        client_pubkey: &PublicKey,
        request_id: &str,
        response: NostrConnectResponse,
    ) -> Result<()> {
        let msg = NostrConnectMessage::response(request_id, response);

        let encrypted = nip44::encrypt(
            self.signer_key.secret_key(),
            client_pubkey,
            msg.as_json(),
            nip44::Version::default(),
        )?;

        let event = EventBuilder::new(Kind::NostrConnect, encrypted)
            .tag(Tag::public_key(*client_pubkey))
            .sign_with_keys(&self.signer_key)?;

        self.pool.send_event(&event).await?;

        println!("📤 Response sent\n");

        Ok(())
    }

    fn should_approve(&self, client_pubkey: &PublicKey, request: &NostrConnectRequest) -> bool {
        match request {
            NostrConnectRequest::Connect {
                public_key: req_pk, ..
            } => {
                println!("\n🔔 New connection request!");
                println!("   From: {}", client_pubkey);
                println!("   App pubkey: {}", req_pk);

                Confirm::new()
                    .with_prompt("Approve connection?")
                    .default(false)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::GetPublicKey => {
                println!("🔑 Request to get public key from {}", client_pubkey);
                true
            }
            NostrConnectRequest::SignEvent(event) => {
                println!("\n📝 Request to sign event:");
                println!("   From: {}", client_pubkey);
                println!("   Kind: {}", event.kind);
                println!(
                    "   Content: {}",
                    if event.content.len() > 100 {
                        format!("{}...", &event.content[..100])
                    } else {
                        event.content.clone()
                    }
                );

                Confirm::new()
                    .with_prompt("Sign this event?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip04Encrypt {
                public_key: target,
                text,
            } => {
                println!("\n🔐 Request to encrypt (NIP-04):");
                println!("   From: {}", client_pubkey);
                println!("   To: {}", target);
                println!(
                    "   Text: {}",
                    if text.len() > 50 {
                        format!("{}...", &text[..50])
                    } else {
                        text.clone()
                    }
                );

                Confirm::new()
                    .with_prompt("Encrypt?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip04Decrypt {
                public_key: from,
                ciphertext,
            } => {
                println!("\n🔓 Request to decrypt (NIP-04):");
                println!("   From: {}", client_pubkey);
                println!("   From pubkey: {}", from);
                println!(
                    "   Ciphertext: {}...",
                    &ciphertext[..ciphertext.len().min(50)]
                );

                Confirm::new()
                    .with_prompt("Decrypt?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip44Encrypt {
                public_key: target,
                text,
            } => {
                println!("\n🔐 Request to encrypt (NIP-44):");
                println!("   From: {}", client_pubkey);
                println!("   To: {}", target);
                println!(
                    "   Text: {}",
                    if text.len() > 50 {
                        format!("{}...", &text[..50])
                    } else {
                        text.clone()
                    }
                );

                Confirm::new()
                    .with_prompt("Encrypt?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Nip44Decrypt {
                public_key: from,
                ciphertext,
            } => {
                println!("\n🔓 Request to decrypt (NIP-44):");
                println!("   From: {}", client_pubkey);
                println!("   From pubkey: {}", from);
                println!(
                    "   Ciphertext: {}...",
                    &ciphertext[..ciphertext.len().min(50)]
                );

                Confirm::new()
                    .with_prompt("Decrypt?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            }
            NostrConnectRequest::Ping => {
                println!("🏓 Ping received from {}", client_pubkey);
                true
            }
        }
    }
}
