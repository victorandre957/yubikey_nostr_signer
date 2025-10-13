# Nostr Bunker com YubiKey

Um bunker Nostr (NIP-46) que usa a YubiKey como armazenamento seguro para chaves privadas, carregando-as **sob demanda** apenas quando necessário.

## 🔐 Arquitetura de Segurança

### Princípio Principal
**A chave privada passa o MENOR tempo possível fora da YubiKey.**

### Como funciona

1. **Armazenamento**: Chave privada Nostr é armazenada no `largeBlob` da YubiKey
2. **Carregamento sob demanda**: Chave é lida da YubiKey SOMENTE quando precisa assinar
3. **Limpeza imediata**: Após a assinatura, a chave é imediatamente descartada da memória
4. **PIN obrigatório**: Cada leitura da YubiKey requer o PIN (com cache temporário do FIDO2)

### Comparação com outras implementações

| Implementação | Segurança da Chave |
|--------------|-------------------|
| **Nostr Bunker padrão** | Chave fica na memória durante toda a execução |
| **YubiKey Bunker** | ✅ Chave carregada SOB DEMANDA, limpa após cada uso |

## 📦 Pré-requisitos

### 1. YubiKey com suporte FIDO2
- YubiKey 5 Series ou superior
- Com suporte a largeBlob

### 2. Chave Nostr salva na YubiKey

Primeiro, você precisa salvar sua chave privada na YubiKey:

```bash
# Inicia o programa de gerenciamento de blobs
cargo run

# Escolha a opção: Write Data to YubiKey
# - Entre com o ID: nostr_key
# - Cole sua chave privada em hexadecimal (64 caracteres)
# - Insira o PIN quando solicitado
```

**⚠️ IMPORTANTE**: A entrada deve ser com ID `nostr_key` para o bunker encontrar a chave.

## 🚀 Executando o Bunker

### 1. Compile e execute

```bash
cargo run --bin yubikey_bunker
```

### 2. O que acontece

1. **Conecta à YubiKey**: Busca dispositivos FIDO2 conectados
2. **Solicita PIN**: Para verificar acesso ao largeBlob
3. **Gera chave temporária**: Para o protocolo NIP-46 (essa NÃO é sua chave real)
4. **Exibe URI de conexão**: Para compartilhar com aplicativos Nostr
5. **Aguarda requisições**: Conecta aos relays e espera por operações

### 3. Saída esperada

```
🚀 Nostr Bunker com YubiKey (NIP-46)

============================================================

📡 Relays configurados:
   • wss://relay.damus.io
   • wss://nos.lol
   • wss://relay.nostr.band

============================================================

🔐 Configurando YubiKey...
Found device: Yubico YubiKey FIDO+CCID 00 00
Credential ID carregado: 32 bytes

✅ YubiKey configurada com sucesso

🔑 Gerando chave temporária para NIP-46...
   • Esta chave é APENAS para o protocolo Nostr Connect
   • Sua chave REAL está segura na YubiKey

🌐 Bunker iniciado!
   URI de conexão: bunker://pubkey123...@relay.damus.io?relay=wss://relay.damus.io&secret=token

============================================================

💡 Como usar:
   1. Compartilhe o URI acima com aplicativos Nostr
   2. Aprove as requisições quando aparecerem
   3. A chave será lida da YubiKey para cada operação
   4. Pressione Ctrl+C para encerrar

🔒 Segurança:
   • Chave privada NUNCA sai da YubiKey permanentemente
   • Carregada SOB DEMANDA para cada assinatura
   • Limpa da memória IMEDIATAMENTE após uso
   • PIN necessário para cada leitura

============================================================
```

## 🔄 Fluxo de Operação

### Quando um app Nostr quer assinar um evento:

1. **App envia requisição** → Relay → Bunker
2. **Bunker pergunta**: "Deseja assinar este evento? (yes/no)"
3. Se você aceitar:
   - 🔐 **Lê chave da YubiKey** (solicita PIN se necessário)
   - ✍️ **Assina o evento** usando a chave
   - 🗑️ **Descarta a chave** imediatamente da memória
   - 📤 **Envia evento assinado** para o relay

### Exemplo de log durante assinatura:

```
📩 Nova requisição de: npub123...

SignEvent:
   Kind: 1
   Content: "Hello Nostr!"
   Tags: []

Aprovar esta requisição? (yes/no): yes
✅ Requisição aprovada

📝 Assinando evento com YubiKey...
🔐 Lendo chave privada da YubiKey...
   [PIN solicitado no terminal]
✅ Chave carregada com sucesso
   Pubkey: npub1abc...

✅ Evento assinado com sucesso
   ID: note1xyz...

✅ Resposta enviada com sucesso
   Event ID: ev123...
```

## 📱 Conectando Apps

### 1. Copie o URI do bunker

Formato: `bunker://pubkey@relay?relay=wss://...&secret=token`

### 2. Cole no app Nostr

Apps que suportam NIP-46:
- **Amethyst** (Android)
- **Damus** (iOS)
- **Nostrudel** (Web)
- **Snort** (Web)
- Qualquer app com suporte a "Nostr Connect" ou "Remote Signer"

### 3. Aprove as requisições no terminal

Sempre que o app quiser fazer algo, você verá no terminal e pode aprovar/rejeitar.

## 🛡️ Segurança em Detalhes

### Duas Chaves Diferentes

1. **Chave Temporária (NIP-46)**
   - Gerada aleatoriamente na inicialização
   - Usada APENAS para criptografia do protocolo NIP-46
   - Perdida quando o bunker encerra
   - **NÃO é sua identidade Nostr**

2. **Chave Real (na YubiKey)**
   - Sua chave privada Nostr verdadeira
   - Armazenada no largeBlob da YubiKey
   - Lida SOB DEMANDA para assinar
   - Limpa da memória após uso

### Proteções Implementadas

- ✅ Chave nunca fica residente na memória
- ✅ Drop automático após cada uso
- ✅ PIN necessário para acesso à YubiKey
- ✅ Aprovação manual para cada operação
- ✅ Logs claros de todas as operações

## 🔧 Configuração Avançada

### Mudando os Relays

Edite `src/bin/yubikey_bunker.rs`:

```rust
let relays = vec![
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band",
    // Adicione mais relays aqui
];
```

### Mudando o Secret Token

```rust
let secret = Some("seu-token-secreto-aqui".to_string());
```

Ou deixe `None` para não ter secret (menos seguro).

### Aprovação Automática (não recomendado)

Para fins de teste, você pode modificar `should_approve()` em `src/yubikey_bunker.rs`:

```rust
async fn should_approve(&self, request: &NostrConnectRequest) -> bool {
    // WARNING: Isto aprova TUDO automaticamente!
    true
}
```

## 🐛 Troubleshooting

### YubiKey não encontrada

```
Error: Nenhum dispositivo FIDO2 encontrado
```

**Soluções:**
- Conecte a YubiKey
- Verifique permissões USB: `sudo usermod -aG plugdev $USER`
- Logout e login novamente

### PIN incorreto

```
Error: PIN verification failed
```

**Soluções:**
- Digite o PIN correto
- Se esqueceu o PIN, use o YubiKey Manager para resetar (⚠️ perde todos os dados)

### Entrada não encontrada

```
Error: Entry 'nostr_key' not found
```

**Soluções:**
- Salve sua chave com ID `nostr_key` usando `cargo run` → Write Data
- Verifique se a chave foi salva corretamente com Read Data

### Chave inválida

```
Error: Falha ao parsear chave privada
```

**Soluções:**
- A chave deve estar em formato hexadecimal (64 caracteres)
- Exemplo: `3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d`

## 📚 Referências

- [NIP-46: Nostr Connect](https://github.com/nostr-protocol/nips/blob/master/46.md)
- [NIP-44: Encrypted Direct Message (versioned)](https://github.com/nostr-protocol/nips/blob/master/44.md)
- [rust-nostr Documentation](https://docs.rs/nostr/)
- [YubiKey FIDO2 Documentation](https://developers.yubico.com/FIDO2/)

## 🔐 Dicas de Segurança

1. **Nunca compartilhe sua chave privada** - Ela deve existir SOMENTE na YubiKey
2. **Use um PIN forte** - Proteja o acesso à YubiKey
3. **Backup da chave** - Mantenha um backup seguro offline (caso perca a YubiKey)
4. **Verifique requisições** - Sempre leia com atenção antes de aprovar
5. **Secret token** - Use um token seguro e compartilhe apenas com apps confiáveis

## 🤝 Contribuindo

Este é um projeto experimental. Sugestões e melhorias são bem-vindas!

## ⚖️ Licença

MIT License - veja LICENSE para detalhes.
