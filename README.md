# 🔐 YubiKey Nostr Signer

**Nostr Bunker (NIP-46) seguro com YubiKey - Suas chaves privadas nunca saem do hardware**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Nostr](https://img.shields.io/badge/nostr-NIP--46-purple.svg)](https://github.com/nostr-protocol/nips/blob/master/46.md)

Um Remote Signer Nostr que armazena chaves privadas com segurança em uma YubiKey, carregando-as apenas sob demanda e limpando a memória imediatamente após cada operação.

## ✨ Features

- 🔒 **Hardware Security**: Chaves armazenadas no largeBlob da YubiKey com criptografia FIDO2
- ⚡ **On-Demand Loading**: Chaves carregadas apenas quando necessárias
- 🧹 **Immediate Cleanup**: Memória zerada automaticamente após cada operação
- 🔐 **PIN Protection**: Todas as operações protegidas por PIN da YubiKey
- 📡 **NIP-46 Compliant**: Implementação completa do protocolo Nostr Connect
- 💬 **NIP-04 & NIP-44**: Suporte completo para mensagens diretas
- 🎯 **User Approval**: Aprovação interativa para cada assinatura
- 🛡️ **Memory Safe**: Escrito em Rust para segurança máxima

## 📁 File Structure

```
yubikey_fido2_teste/
├── src/
│   ├── main.rs              # Menu principal integrado (gerenciar chaves + iniciar bunker)
│   ├── lib.rs               # Library exports e tipos comuns
│   ├── auth.rs              # Utilitários de autenticação PIN
│   ├── device.rs            # Detecção e inicialização de dispositivos
│   ├── credential.rs        # Gerenciamento de credenciais FIDO2
│   ├── encryption.rs        # Criptografia AES-GCM
│   ├── blob_operations.rs   # Hub centralizado para operações largeBlob (8 funções públicas)
│   ├── yubikey_keys.rs      # YubiKey Key Manager com carregamento sob demanda
│   └── yubikey_bunker.rs    # Implementação Nostr Bunker (NIP-46) com YubiKey
├── Cargo.toml              # Dependências e configuração do projeto
├── README.md               # Esta documentação
└── LICENSE                 # Licença MIT
```

## 🆕 Nostr Bunker com YubiKey (NIP-46)

Este projeto implementa um **Nostr Bunker seguro com YubiKey** seguindo a [NIP-46](https://github.com/nostr-protocol/nips/blob/master/46.md).

### O que é um Nostr Bunker?

Um Nostr Bunker é um signer remoto que mantém suas chaves privadas seguras e permite que aplicativos solicitem assinatura de eventos sem ter acesso direto às chaves. **Nesta implementação, as chaves ficam armazenadas com segurança na YubiKey e são carregadas apenas sob demanda para cada operação.**

### 🔐 Segurança Diferenciada

- **Carregamento sob demanda**: A chave privada é carregada da YubiKey apenas quando necessária
- **Memória limpa**: Após cada operação, a chave é imediatamente removida da memória usando `zeroize`
- **Proteção por PIN**: Todas as operações requerem autenticação PIN da YubiKey
- **Armazenamento seguro**: Chaves criptografadas no largeBlob da YubiKey

### Quick Start

**Executar o aplicativo (menu integrado):**

```bash
cargo run
```

O menu oferece duas opções:
1. **Gerenciar chaves da YubiKey** - Criar, listar, deletar chaves armazenadas
2. **Iniciar Nostr Bunker** - Iniciar o servidor de assinatura remota

### Operações NIP-46 Suportadas

- ✅ **connect** - Conexão de clientes via Nostr Connect URI
- ✅ **sign_event** - Assinatura de eventos Nostr
- ✅ **nip04_encrypt** - Encriptação NIP-04 (DM legado)
- ✅ **nip04_decrypt** - Decriptação NIP-04
- ✅ **nip44_encrypt** - Encriptação NIP-44 (DM moderno)
- ✅ **nip44_decrypt** - Decriptação NIP-44
- ✅ **get_public_key** - Obter chave pública
- ✅ Aprovação interativa do usuário para cada operação

## 🔧 Module Details

### `main.rs` - Menu Principal Integrado

- **Purpose**: Ponto de entrada único com menu interativo
- **Key Functions**:
  - `main()`: Loop principal do menu
  - `manage_keys()`: Gerenciamento de chaves (criar, listar, deletar)
  - `start_bunker()`: Inicializa o Nostr Bunker com YubiKey
- **Menu Options**:
  1. Gerenciar chaves da YubiKey
  2. Iniciar Nostr Bunker
  3. Sair

### `yubikey_bunker.rs` - Nostr Bunker com YubiKey

- **Purpose**: Implementação completa do protocolo NIP-46 usando YubiKey
- **Key Components**:
  - `YubikeyNostrBunker`: Servidor bunker principal
  - Gerenciamento manual de relay pool
  - Sistema de aprovação interativa do usuário
- **NIP-46 Methods**:
  - `connect`: Estabelece conexão com cliente
  - `sign_event`: Assina eventos Nostr
  - `nip04_encrypt/decrypt`: Mensagens diretas (legado)
  - `nip44_encrypt/decrypt`: Mensagens diretas (moderno)
  - `get_public_key`: Retorna chave pública
- **Security**: Carrega chave apenas para cada operação, limpa memória imediatamente

### `yubikey_keys.rs` - YubiKey Key Manager

- **Purpose**: Gerenciador de chaves com carregamento sob demanda
- **Key Components**:
  - `YubikeyKeyManager`: Gerenciador principal
  - Cache da chave pública (não sensível)
  - Carregamento sob demanda da chave privada
- **Key Functions**:
  - `new()`: Inicializa e seleciona entrada do blob
  - `load_private_key()`: Carrega chave privada temporariamente
  - `with_key()`: Executa operação com chave, depois limpa
  - `public_key()`: Retorna chave pública (cached)
- **Security Pattern**: 
  ```rust
  // Chave carregada apenas durante a operação
  manager.with_key(|keys| {
      // usa keys aqui
  })?; // keys é automaticamente dropada e zerada
  ```

### `blob_operations.rs` - Hub Centralizado de Operações

- **Purpose**: Centraliza TODAS as operações largeBlob da YubiKey
- **Public Functions** (8 funções reutilizáveis):
  - `select_and_read_entry()`: Seleção interativa de entrada
  - `read_blob_entry_by_index()`: Leitura direta por índice
  - `decrypt_entry_raw()`: Descriptografia de entrada
  - `get_blob_content()`: Obtém conteúdo do blob
  - `parse_blob_entries()`: Parse de entradas
  - `encrypt_data()`: Criptografia AES-GCM
  - `decrypt_data()`: Descriptografia AES-GCM
  - `write_blob()`: Escrita de entradas criptografadas
- **Architecture**: Todas as outras funções USAM estas, sem reimplementação

### `encryption.rs` - Criptografia

- **Purpose**: Implementação de criptografia AES-GCM
- **Features**:
  - AES-256-GCM (Galois/Counter Mode)
  - Nonces aleatórios de 96 bits
  - Tag de autenticação de 128 bits
- **Security**: Criptografia autenticada, previne adulteração

### `credential.rs` - Gerenciamento de Credenciais FIDO2

- **Purpose**: Criação e gerenciamento de credenciais FIDO2
- **Key Functions**:
  - `get_credential_id()`: Cria credenciais residentes com HMAC-secret
  - `get_hmac_secret()`: Deriva chaves de criptografia do dispositivo
- **FIDO2 Features**:
  - Resident keys para armazenamento persistente
  - Extensão HMAC-secret para derivação de chaves

### `auth.rs` - Utilitários de Autenticação

- **Purpose**: Autenticação PIN para operações FIDO2
- **Key Functions**:
  - `get_pin()`: Input seguro de PIN (caracteres ocultos)
- **Security**: Usa `rpassword` para entrada segura

### `device.rs` - Gerenciamento de Dispositivos

- **Purpose**: Detecção e inicialização de dispositivos FIDO2
- **Key Functions**:
  - Enumeração e conexão de dispositivos
  - Validação de compatibilidade de hardware

## 🚀 Getting Started

### Prerequisites

- **Hardware**: Dispositivo compatível com FIDO2 (YubiKey 5 series recomendado)
- **Software**: Rust 1.70+ com Cargo
- **YubiKey**: Firmware 5.2.3+ com suporte a largeBlob

### Installation

1. **Clone o repositório**:

   ```bash
   git clone https://github.com/victorandre957/yubikey_nostr_signer.git
   cd yubikey_nostr_signer
   ```

2. **Compile a aplicação**:

   ```bash
   cargo build --release
   ```

3. **Execute a aplicação**:

   ```bash
   cargo run
   ```

### Primeira Execução

1. Conecte sua YubiKey
2. A aplicação detectará e inicializará automaticamente seu dispositivo
3. Digite o PIN do dispositivo quando solicitado
4. O sistema criará uma credencial se nenhuma existir
5. Escolha uma opção do menu:
   - **Opção 1**: Gerenciar chaves (criar, listar, deletar)
   - **Opção 2**: Iniciar o Nostr Bunker

### Workflow Típico

1. **Criar uma chave Nostr** (primeira vez):
   - Menu Principal → 1 (Gerenciar chaves)
   - Submenu → 1 (Criar nova chave)
   - Digite um ID memorável (ex: "main-key")
   
2. **Iniciar o bunker**:
   - Menu Principal → 2 (Iniciar Nostr Bunker)
   - Selecione a chave criada
   - Copie o Nostr Connect URI gerado
   
3. **Conectar um cliente**:
   - Cole o URI no seu aplicativo Nostr favorito
   - Aprove as requisições de assinatura no terminal

## 💡 Usage Examples

### 1. Gerenciamento de Chaves

**Criar uma nova chave Nostr:**

```text
Menu Principal:
1. Gerenciar chaves da YubiKey
2. Iniciar Nostr Bunker
3. Sair

Escolha uma opção: 1

=== Gerenciamento de Chaves da YubiKey ===
1. Criar nova chave Nostr
2. Listar chaves armazenadas
3. Ler chave específica
4. Deletar chave
5. Voltar ao menu principal

Escolha: 1
Digite um ID para esta entrada: my-nostr-key
✓ Par de chaves Nostr gerado e armazenado com sucesso!
Chave pública: npub1...
```

**Listar chaves armazenadas:**

```text
Escolha: 2

Entradas existentes no blob:
1: my-nostr-key
2: backup-key
3: bot-key
```

### 2. Usando o Nostr Bunker

**Iniciar o bunker:**

```text
Menu Principal:
1. Gerenciar chaves da YubiKey
2. Iniciar Nostr Bunker
3. Sair

Escolha uma opção: 2

Entradas existentes no blob:
1: my-nostr-key
2: backup-key
3: bot-key

Digite o número da entrada para usar (ou 0 para cancelar): 1

✓ YubiKey Key Manager inicializado!
Chave pública do bunker: npub1...

🔗 Nostr Connect URI:
bunker://npub1...?relay=wss://relay.damus.io&relay=wss://nos.lol

📋 Compartilhe este URI com o cliente que deseja conectar
🔐 Aguardando conexões...
```

**Aprovar assinatura de evento:**

```text
🔔 Nova requisição de assinatura!

Cliente: npub1abc...
Tipo de evento: 1 (nota)
Conteúdo: "Hello Nostr!"
Tags: []

Aprovar esta assinatura? (s/n): s
Digite seu PIN: ****

✅ Evento assinado e enviado!
```

### 3. Conectando um Cliente

Qualquer cliente Nostr que suporte NIP-46 pode se conectar usando o URI do bunker:

```javascript
// Exemplo JavaScript (usando nostr-tools ou similar)
const bunkerURI = "bunker://npub1...?relay=wss://relay.damus.io";
const signer = await NostrConnect.connect(bunkerURI);

// Agora todas as assinaturas serão feitas via bunker
const event = await signer.signEvent({
  kind: 1,
  content: "Signed remotely!",
  tags: [],
  created_at: Math.floor(Date.now() / 1000)
});
```

## 🔒 Security Features

### Hardware-Backed Security

- **YubiKey Storage**: Chaves armazenadas com segurança no largeBlob da YubiKey
- **FIDO2 HMAC-secret**: Chaves de criptografia nunca saem do hardware
- **PIN Protection**: Todas as operações requerem autenticação PIN
- **Resident Keys**: Credenciais armazenadas com segurança no dispositivo

### Encryption Standards

- **AES-256-GCM**: Criptografia autenticada de padrão industrial
- **Random Nonces**: Cada criptografia usa nonce aleatório único de 96 bits
- **Authentication Tags**: Tags de 128 bits previnem adulteração de dados
- **Salt-based Derivation**: HMAC-secret usa salt aleatório para cada derivação

### On-Demand Key Loading

- **Minimal Exposure**: Chave privada carregada apenas quando necessária
- **Immediate Cleanup**: Memória zerada com `zeroize` após cada operação
- **No Persistence**: Chaves nunca são armazenadas em disco ou memória permanente
- **Operation Pattern**:
  ```rust
  // Chave existe apenas dentro do closure
  manager.with_key(|keys| {
      let signature = keys.sign_event(...)?;
      Ok(signature)
  })?; // keys automaticamente dropada e zerada aqui
  ```

### Data Protection

- **Memory Safety**: Sistema de ownership do Rust previne buffer overflows
- **Zeroize**: Biblioteca `zeroize` garante limpeza criptográfica da memória
- **No Key Caching**: Chave privada nunca é mantida em cache
- **User Approval**: Aprovação interativa para cada operação de assinatura


## 🛠️ Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ctap-hid-fido2` | 3.5.5 | Implementação do protocolo FIDO2 |
| `nostr` | 0.43 | Biblioteca Nostr (NIP-04, NIP-44, NIP-46) |
| `nostr-connect` | 0.43 | Implementação Nostr Connect |
| `nostr-relay-pool` | 0.43 | Gerenciamento de pool de relays |
| `aes-gcm` | 0.10 | Criptografia AES-GCM autenticada |
| `tokio` | 1.0 | Runtime assíncrono |
| `dialoguer` | 0.12 | Interface de usuário interativa |
| `zeroize` | 1.8 | Limpeza segura de memória |
| `hex` | 0.4 | Codificação/decodificação hexadecimal |
| `base64` | 0.22 | Codificação Base64 |
| `rand` | 0.9 | Geração de números aleatórios criptográficos |
| `anyhow` | 1.0 | Tratamento de erros |
| `rpassword` | 7.3 | Input seguro de senha/PIN |
| `tracing` | 0.1 | Logging e tracing |

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/new-feature`
3. Commit your changes: `git commit -am 'Add new feature'`
4. Push to the branch: `git push origin feat/new-feature`
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Security Considerations

- **PIN Protection**: Sempre use PIN na sua YubiKey
- **Firmware Atualizado**: Mantenha o firmware da YubiKey atualizado
- **Backup de Chaves**: Considere ter uma YubiKey backup com as mesmas chaves
- **PIN Confidencial**: Nunca compartilhe seu PIN
- **Aprovação Consciente**: Revise cuidadosamente cada requisição antes de aprovar
- **Ambiente Seguro**: Execute o bunker em um ambiente confiável
- **Relay Confiável**: Use apenas relays confiáveis na URI de conexão
- **Autenticidade**: Verifique a autenticidade do dispositivo antes de usar

## 🔗 References

### Nostr Protocol

- [NIP-01: Basic Protocol](https://github.com/nostr-protocol/nips/blob/master/01.md)
- [NIP-04: Encrypted Direct Messages (legacy)](https://github.com/nostr-protocol/nips/blob/master/04.md)
- [NIP-44: Encrypted Direct Messages](https://github.com/nostr-protocol/nips/blob/master/44.md)
- [NIP-46: Nostr Connect (Remote Signer)](https://github.com/nostr-protocol/nips/blob/master/46.md)
- [rust-nostr Documentation](https://docs.rs/nostr/)

### FIDO2 & Security

- [FIDO2 Specification](https://fidoalliance.org/specs/fido-v2.1-ps-20210615/fido-client-to-authenticator-protocol-v2.1-ps-errata-20220621.html)
- [WebAuthn HMAC-secret Extension](https://w3c.github.io/webauthn/#sctn-hmac-secret-extension)
- [YubiKey FIDO2 Developer Guide](https://developers.yubico.com/FIDO2/)
- [ctap-hid-fido2 Documentation](https://docs.rs/ctap-hid-fido2/)

---

## 🎯 Project Status

✅ **Pronto para produção**

- [x] Implementação completa NIP-46
- [x] Integração segura com YubiKey
- [x] Carregamento sob demanda de chaves
- [x] Limpeza automática de memória
- [x] Suporte completo NIP-04 e NIP-44
- [x] Interface de usuário interativa
- [x] Zero duplicação de código
- [x] Zero warnings de compilação
- [x] Documentação completa

---

**Feito com 🔐 para gerenciamento seguro de chaves Nostr**
