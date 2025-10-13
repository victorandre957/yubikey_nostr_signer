# Exemplos de Uso do Nostr Bunker

## 🚀 Exemplo 1: Servidor Básico

```rust
use anyhow::Result;
use nostr::prelude::*;
use yubikey_fido2_teste::NostrBunker;

#[tokio::main]
async fn main() -> Result<()> {
    // Chaves do bunker
    let signer_key = Keys::generate();
    let user_key = Keys::generate();
    
    // Relays
    let relays = vec!["wss://relay.damus.io", "wss://nos.lol"];
    
    // Criar e iniciar bunker
    let bunker = NostrBunker::new(signer_key, user_key, relays, None)?;
    println!("URI: {}", bunker.bunker_uri());
    bunker.serve().await?;
    
    Ok(())
}
```

## 🔌 Exemplo 2: Cliente Conectando ao Bunker

```rust
use anyhow::Result;
use nostr::prelude::*;
use nostr_connect::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let bunker_uri = "bunker://79dff...?relay=wss://relay.damus.io";
    let app_keys = Keys::generate();
    
    let signer = NostrConnect::new(
        NostrConnectURI::parse(bunker_uri)?,
        app_keys,
        Duration::from_secs(120),
        None,
    )?;
    
    // Obter chave pública
    let pubkey = signer.get_public_key().await?;
    println!("Pubkey: {}", pubkey.to_bech32()?);
    
    // Assinar evento
    let event = EventBuilder::text_note("Hello!")
        .sign(&signer)
        .await?;
    println!("Event ID: {}", event.id);
    
    Ok(())
}
```

## 📱 Exemplo 3: Integração com Aplicativo Nostr

```rust
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Conectar ao bunker como signer
    let uri = NostrConnectURI::parse("bunker://...")?;
    let app_keys = Keys::generate();
    let signer = NostrConnect::new(uri, app_keys, Duration::from_secs(120), None)?;
    
    // Criar cliente com o bunker como signer
    let client = Client::new(signer);
    
    // Adicionar relays
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nos.lol").await?;
    client.connect().await;
    
    // Publicar eventos
    client.publish_text_note("Hello from bunker!").await?;
    
    // Enviar mensagem privada
    let receiver = PublicKey::from_bech32("npub1...")?;
    client.send_private_msg(receiver, "Secret message").await?;
    
    Ok(())
}
```

## 🔐 Exemplo 4: Bunker com YubiKey

```rust
use yubikey_fido2_teste::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Encontrar YubiKey
    let mut device = find_fido_device()?;
    let credential_id = get_credential_id(&mut device)?;
    
    // Ler chave da YubiKey
    let key_bytes = read_blob(&mut device, &credential_id)?;
    let user_key = Keys::parse(&hex::encode(key_bytes))?;
    
    // Gerar chave do signer (pode ser armazenada também)
    let signer_key = Keys::generate();
    
    // Criar bunker com chave da YubiKey
    let bunker = NostrBunker::new(
        signer_key,
        user_key,
        vec!["wss://relay.damus.io"],
        Some("yubikey-secret".to_string()),
    )?;
    
    println!("🔐 Bunker protegido por YubiKey!");
    println!("URI: {}", bunker.bunker_uri());
    bunker.serve().await?;
    
    Ok(())
}
```

## 🧪 Exemplo 5: Testes Automatizados

```rust
#[tokio::test]
async fn test_bunker_sign_event() -> Result<()> {
    // Criar bunker
    let signer_key = Keys::generate();
    let user_key = Keys::generate();
    let bunker = NostrBunker::new(
        signer_key,
        user_key,
        vec!["wss://relay.damus.io"],
        Some("test-secret".to_string()),
    )?;
    
    // Iniciar bunker em background
    tokio::spawn(async move {
        bunker.serve().await.ok();
    });
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Conectar cliente
    let app_keys = Keys::generate();
    let uri = bunker.bunker_uri();
    let signer = NostrConnect::new(uri, app_keys, Duration::from_secs(30), None)?;
    
    // Testar assinatura
    let pubkey = signer.get_public_key().await?;
    let event = EventBuilder::text_note("Test")
        .build(pubkey);
    let signed = signer.sign_event(event).await?;
    
    assert!(signed.verify().is_ok());
    
    Ok(())
}
```

## 🔄 Exemplo 6: Rotação de Chaves

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Chave antiga (da YubiKey)
    let old_key = Keys::parse("hex-or-nsec...")?;
    
    // Criar nova chave
    let new_key = Keys::generate();
    
    // Migrar: publicar evento delegando assinatura
    let client = Client::new(old_key.clone());
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;
    
    // Criar evento de delegação (NIP-26)
    let delegation = EventBuilder::delegation(
        &old_key,
        new_key.public_key(),
        Timestamp::now(),
        None, // Sem expiração
    )?;
    client.send_event_builder(delegation).await?;
    
    // Agora usar nova chave no bunker
    let bunker = NostrBunker::new(
        Keys::generate(),
        new_key,
        vec!["wss://relay.damus.io"],
        None,
    )?;
    
    bunker.serve().await?;
    
    Ok(())
}
```

## 🌐 Exemplo 7: Múltiplos Bunkers

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Bunker pessoal (YubiKey)
    let personal_key = read_key_from_yubikey()?;
    let personal_bunker = NostrBunker::new(
        Keys::generate(),
        personal_key,
        vec!["wss://relay.damus.io"],
        Some("personal".to_string()),
    )?;
    
    // Bunker profissional (outro device)
    let work_key = Keys::parse("nsec1...")?;
    let work_bunker = NostrBunker::new(
        Keys::generate(),
        work_key,
        vec!["wss://relay.nostr.band"],
        Some("work".to_string()),
    )?;
    
    // Executar em paralelo
    tokio::try_join!(
        personal_bunker.serve(),
        work_bunker.serve(),
    )?;
    
    Ok(())
}
```

## 🔧 Exemplo 8: Bunker com Configuração Customizada

```rust
use nostr_connect::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let keys = NostrConnectKeys {
        signer: Keys::generate(),
        user: Keys::parse("nsec1...")?,
    };
    
    // Criar signer customizado
    let signer = NostrConnectRemoteSigner::new(
        keys,
        vec!["wss://relay.damus.io"],
        Some("my-secret".to_string()),
        None,
    )?;
    
    println!("URI: {}", signer.bunker_uri());
    
    // Servir com ações customizadas
    struct MyActions;
    
    impl NostrConnectSignerActions for MyActions {
        fn approve(&self, _: &PublicKey, req: &NostrConnectRequest) -> bool {
            match req {
                NostrConnectRequest::GetPublicKey => true,
                NostrConnectRequest::Ping => true,
                NostrConnectRequest::SignEvent(event) => {
                    // Só aprovar eventos kind 1 (text note)
                    event.kind == Kind::TextNote
                }
                _ => false, // Rejeitar resto
            }
        }
    }
    
    signer.serve(MyActions).await?;
    
    Ok(())
}
```

## 📊 Exemplo 9: Logging e Monitoramento

```rust
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_thread_ids(true)
        .init();
    
    info!("Iniciando Nostr Bunker...");
    
    let bunker = NostrBunker::new(
        Keys::generate(),
        Keys::generate(),
        vec!["wss://relay.damus.io"],
        None,
    )?;
    
    info!("Bunker URI: {}", bunker.bunker_uri());
    
    // Servir com logs
    bunker.serve().await?;
    
    Ok(())
}
```

## 🎯 Exemplo 10: CLI Completo

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Usar chave da YubiKey
    #[arg(long)]
    yubikey: bool,
    
    /// Ou fornecer nsec diretamente
    #[arg(long)]
    nsec: Option<String>,
    
    /// Relays (pode repetir)
    #[arg(long, default_value = "wss://relay.damus.io")]
    relay: Vec<String>,
    
    /// Segredo para autorização
    #[arg(long)]
    secret: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let user_key = if args.yubikey {
        let mut device = find_fido_device()?;
        let cred = get_credential_id(&mut device)?;
        let bytes = read_blob(&mut device, &cred)?;
        Keys::parse(&hex::encode(bytes))?
    } else if let Some(nsec) = args.nsec {
        Keys::parse(&nsec)?
    } else {
        Keys::generate()
    };
    
    let bunker = NostrBunker::new(
        Keys::generate(),
        user_key,
        args.relay,
        args.secret,
    )?;
    
    println!("🚀 Bunker iniciado!");
    println!("URI: {}", bunker.bunker_uri());
    
    bunker.serve().await?;
    
    Ok(())
}
```

## 💡 Dicas

1. **Sempre valide eventos** antes de assinar
2. **Use secrets fortes** para autorização automática
3. **Conecte a múltiplos relays** para redundância
4. **Implemente rate limiting** em produção
5. **Faça backup das chaves** de forma segura
6. **Use logging** para auditoria
7. **Teste em testnet** antes de produção
