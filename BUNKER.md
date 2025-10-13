# Nostr Bunker - Signer Remoto NIP-46

Este projeto implementa um **Nostr Bunker** seguindo a [NIP-46](https://github.com/nostr-protocol/nips/blob/master/46.md), permitindo assinar eventos Nostr de forma remota e segura.

## 🎯 O que é um Bunker?

Um Nostr Bunker é um signer remoto que:
- Mantém suas chaves privadas seguras em um local separado
- Permite que aplicativos solicitem assinatura de eventos sem ter acesso direto às chaves
- Requer aprovação do usuário para cada operação sensível
- Suporta encriptação/decriptação NIP-04 e NIP-44

## 🚀 Como Usar

### 1. Iniciar o Servidor Bunker

```bash
cargo run --bin bunker
```

Isso irá:
1. Gerar chaves para o bunker
2. Conectar aos relays configurados
3. Exibir um URI `bunker://...` que você pode compartilhar com aplicativos

Exemplo de saída:
```
🚀 Iniciando Nostr Bunker (NIP-46)...

📌 Chaves geradas:
   Signer pubkey: npub1...
   User pubkey: npub1...

🔗 Compartilhe este URI com clientes:
   bunker://79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3?relay=wss://relay.damus.io&relay=wss://nos.lol&secret=secret-token-123

💡 Dica: Use este URI em aplicativos como Amethyst, Damus, etc.

⏳ Aguardando requisições...
```

### 2. Conectar um Cliente

Em outro terminal, execute o cliente de teste:

```bash
cargo run --bin bunker_client
```

Quando solicitado, cole o URI do bunker exibido no servidor.

O cliente irá:
1. Conectar ao bunker
2. Solicitar a chave pública
3. Assinar um evento de texto
4. Testar encriptação NIP-04 e NIP-44

### 3. Aprovar Requisições

Quando o cliente fizer requisições, o servidor irá perguntar se você deseja aprovar:

```
📝 Solicitação para assinar evento:
   De: npub1...
   Kind: 1
   Content: Hello from Nostr Bunker! 🎉
? Assinar este evento? (y/N)
```

Digite `y` e pressione Enter para aprovar, ou `n` para rejeitar.

## 🔧 Integração com Aplicativos

Você pode usar o URI do bunker em qualquer aplicativo que suporte NIP-46:

### Amethyst (Android)
1. Vá para Configurações > Chaves
2. Escolha "Remote Signer"
3. Cole o URI do bunker
4. Aprove a conexão no servidor bunker

### nak (CLI)
```bash
nak connect <bunker-uri>
```

### Usar como Signer em Código

```rust
use nostr_connect::prelude::*;
use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let uri = NostrConnectURI::parse("bunker://...")?;
    let app_keys = Keys::generate();
    let signer = NostrConnect::new(uri, app_keys, Duration::from_secs(120), None)?;
    
    // Usar o signer
    let event = EventBuilder::text_note("Hello")
        .sign(&signer)
        .await?;
    
    Ok(())
}
```

## 🔐 Operações Suportadas

O bunker suporta as seguintes operações da NIP-46:

- ✅ `connect` - Conectar um novo cliente
- ✅ `get_public_key` - Obter a chave pública
- ✅ `sign_event` - Assinar eventos
- ✅ `nip04_encrypt` - Encriptar mensagens (NIP-04)
- ✅ `nip04_decrypt` - Decriptar mensagens (NIP-04)
- ✅ `nip44_encrypt` - Encriptar mensagens (NIP-44)
- ✅ `nip44_decrypt` - Decriptar mensagens (NIP-44)
- ✅ `ping` - Verificar conectividade

## 🎯 Próximos Passos

### Integração com YubiKey

Em vez de gerar chaves aleatórias, você pode modificar o código para:
1. Ler a chave privada da YubiKey usando o módulo `encryption`
2. Usar essa chave como `user_key` no bunker
3. Armazenar o segredo do bunker na YubiKey também

Exemplo:
```rust
// Em vez de Keys::generate()
let user_key_bytes = read_blob(&mut device, &credential_id)?;
let user_key = Keys::parse(&hex::encode(user_key_bytes))?;
```

### Autorização Automática

Para confiar automaticamente em certos clientes:
```rust
let authorized_pubkeys = vec![
    PublicKey::from_hex("...")?,
];

// Modificar BunkerActions::approve() para verificar
if authorized_pubkeys.contains(public_key) {
    return true; // Aprovar automaticamente
}
```

### Persistência

Salvar chaves autorizadas e configurações:
```rust
// Salvar em arquivo ou na YubiKey
std::fs::write("bunker_config.json", serde_json::to_string(&config)?)?;
```

## 📚 Referências

- [NIP-46: Nostr Connect](https://github.com/nostr-protocol/nips/blob/master/46.md)
- [NIP-04: Encrypted Direct Messages](https://github.com/nostr-protocol/nips/blob/master/04.md)
- [NIP-44: Versioned Encryption](https://github.com/nostr-protocol/nips/blob/master/44.md)
- [rust-nostr Documentation](https://docs.rs/nostr)
- [nostr-connect Crate](https://docs.rs/nostr-connect)

## 🐛 Troubleshooting

**Erro: "failed to connect to relay"**
- Verifique sua conexão de internet
- Tente usar outros relays

**Erro: "timeout waiting for response"**
- Aumente o timeout no cliente:
  ```rust
  NostrConnect::new(uri, keys, Duration::from_secs(300), None)
  ```

**Cliente não recebe respostas**
- Certifique-se de que ambos estão usando os mesmos relays
- Verifique se você aprovou a requisição no servidor
