# 🔐 YubiKey Nostr Bunker

> Nostr Remote Signer (NIP-46) with YubiKey - Your private keys stay secure in hardware

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Nostr](https://img.shields.io/badge/nostr-NIP--46-purple.svg)](https://github.com/nostr-protocol/nips/blob/master/46.md)

A Nostr Remote Signer that stores your private keys encrypted in a YubiKey, allowing you to sign events remotely without exposing your keys.

## 📖 What is this?

This project implements a **Nostr Bunker** following [NIP-46](https://github.com/nostr-protocol/nips/blob/master/46.md). A bunker is a remote signing server that keeps your keys secure while allowing Nostr applications (like Damus, Amethyst, Primal) to request event signatures.

**Key difference:** Your keys are encrypted in the YubiKey and loaded only when needed, being immediately removed from memory after each operation.

## ⚡ Requirements

- **Hardware**: YubiKey 5 Series (firmware 5.2.3+) with FIDO2/largeBlob support
- **Software**: Rust 1.70+ with Cargo
- **System**: Linux, macOS or Windows with USB HID support

## 📁 Project Structure

```text
src/
├── main.rs              # Main menu (manage keys + bunker)
├── yubikey_bunker.rs    # NIP-46 server with YubiKey
├── yubikey_helper.rs    # Key manager (on-demand loading)
├── blob_operations.rs   # Read/write operations on largeBlob
├── encryption.rs        # AES-256-GCM encryption
├── credential.rs        # FIDO2 credential management
├── device.rs            # FIDO2 device detection
└── auth.rs              # Secure PIN input

examples/
└── bunker_client.rs     # NIP-46 test client
```

### Module Descriptions

- **`yubikey_bunker.rs`**: Implements NIP-46 protocol, manages Nostr client connections and processes signing requests
- **`yubikey_helper.rs`**: Manages keys stored in YubiKey, loading them only when needed and cleaning memory immediately
- **`blob_operations.rs`**: Functions to read/write encrypted data in YubiKey's largeBlob
- **`encryption.rs`**: AES-GCM encryption/decryption using YubiKey's HMAC-secret as key
- **`credential.rs`**: Creates and manages FIDO2 resident credentials
- **`device.rs`**: Detects and initializes FIDO2/YubiKey devices
- **`auth.rs`**: Requests user PIN securely

## 🚀 Getting Started

### Prerequisites

- **Hardware**: FIDO2 compatible device (YubiKey 5 series recommended)
- **Software**: Rust 1.70+ with Cargo
- **YubiKey**: Firmware 5.2.3+ with largeBlob support

### Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/victorandre957/yubikey_nostr_signer.git
   cd yubikey_nostr_signer
   ```

2. **Build the application**:

   ```bash
   cargo build --release
   ```

3. **Run the application**:

   ```bash
   cargo run
   ```

### First Run

1. Connect your YubiKey
2. The application will automatically detect and initialize your device
3. Enter the device PIN when prompted
4. The system will create a credential if none exists
5. Choose an option from the menu:
   - **Option 1**: Manage keys (create, list, delete)
   - **Option 2**: Start Nostr Bunker

### Typical Workflow

1. **Create a Nostr key** (first time):
   - Main Menu → 1 (Manage keys)
   - Submenu → 1 (Create new key)
   - Enter a memorable ID (e.g., "main-key")
   
2. **Start the bunker**:
   - Main Menu → 2 (Start Nostr Bunker)
   - Select the created key
   - Copy the generated Nostr Connect URI
   
3. **Connect a client**:
   - Paste the URI into your favorite Nostr app
   - Approve signing requests in the terminal

## 💡 Usage Examples

### 1. Key Management

**Create a new Nostr key:**

```text
Main Menu:
1. Manage Keys
2. Start NIP-46 Bunker
3. Exit

Option (1-3): 1

=== YubiKey Key Management ===
1. Store key
2. Read key
3. Delete key
4. Back

Option (1-4): 1
Enter private key (hex): <your-nostr-key-hex>
✓ Nostr keypair generated and stored successfully!
Public key: npub1...
```

**List stored keys:**

```text
Option: 2

Existing blob entries:
1: my-nostr-key
2: backup-key
3: bot-key
```

### 2. Using the Nostr Bunker

**Start the bunker:**

```text
Main Menu:
1. Manage Keys
2. Start NIP-46 Bunker
3. Exit

Option (1-3): 2

Existing blob entries:
1: my-nostr-key
2: backup-key
3: bot-key

Enter entry number to use (or 0 to cancel): 1

✓ YubiKey Key Manager initialized!
Bunker public key: npub1...

🔗 Nostr Connect URI:
bunker://npub1...?relay=wss://relay.damus.io&relay=wss://nos.lol

📋 Share this URI with the client you want to connect
🔐 Waiting for connections...
```

**Approve event signing:**

```text
🔔 New signing request!

Client: npub1abc...
Event type: 1 (note)
Content: "Hello Nostr!"
Tags: []

Approve this signature? (y/n): y
Enter your PIN: ****

✅ Event signed and sent!
```

### 3. Connecting a Client

Any Nostr client that supports NIP-46 can connect using the bunker URI:

```javascript
// JavaScript example (using nostr-tools or similar)
const bunkerURI = "bunker://npub1...?relay=wss://relay.damus.io";
const signer = await NostrConnect.connect(bunkerURI);

// Now all signatures will be done via bunker
const event = await signer.signEvent({
  kind: 1,
  content: "Signed remotely!",
  tags: [],
  created_at: Math.floor(Date.now() / 1000)
});
```

## 🔒 Security Features

### Hardware-Backed Security

- **YubiKey Storage**: Keys stored securely in YubiKey's largeBlob
- **FIDO2 HMAC-secret**: Encryption keys never leave the hardware
- **PIN Protection**: All operations require PIN authentication
- **Resident Keys**: Credentials stored securely on the device

### Encryption Standards

- **AES-256-GCM**: Industry-standard authenticated encryption
- **Random Nonces**: Each encryption uses unique 96-bit random nonce
- **Authentication Tags**: 128-bit tags prevent data tampering
- **Salt-based Derivation**: HMAC-secret uses random salt for each derivation

### On-Demand Key Loading

- **Minimal Exposure**: Private key loaded only when needed
- **Immediate Cleanup**: Memory zeroed after each operation
- **No Persistence**: Keys never stored on disk or permanent memory
- **Operation Pattern**:

  ```rust
  // Key exists only inside the closure
  manager.with_key(|keys| {
      let signature = keys.sign_event(...)?;
      Ok(signature)
  })?; // keys automatically dropped and zeroed here
  ```

### Data Protection

- **Memory Safety**: Rust's ownership system prevents buffer overflows
- **No Key Caching**: Private key never cached
- **User Approval**: Interactive approval for each signing operation

## 🛠️ Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ctap-hid-fido2` | 3.5.5 | FIDO2 protocol implementation |
| `nostr` | 0.43 | Nostr library (NIP-04, NIP-44, NIP-46) |
| `nostr-connect` | 0.43 | Nostr Connect implementation |
| `nostr-relay-pool` | 0.43 | Relay pool management |
| `aes-gcm` | 0.10 | Authenticated AES-GCM encryption |
| `tokio` | 1.0 | Async runtime |
| `dialoguer` | 0.12 | Interactive user interface |
| `zeroize` | 1.8 | Secure memory cleanup |
| `hex` | 0.4 | Hexadecimal encoding/decoding |
| `base64` | 0.22 | Base64 encoding |
| `rand` | 0.9 | Cryptographic random number generation |
| `anyhow` | 1.0 | Error handling |
| `rpassword` | 7.3 | Secure password/PIN input |
| `tracing` | 0.1 | Logging and tracing |

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/new-feature`
3. Commit your changes: `git commit -am 'Add new feature'`
4. Push to the branch: `git push origin feat/new-feature`
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Made with 🔐 for secure Nostr key management

## ⚠️ Security Considerations

- **PIN Protection**: Always use a PIN on your YubiKey
- **Updated Firmware**: Keep YubiKey firmware updated
- **Key Backup**: Consider having a backup YubiKey with the same keys
- **PIN Confidentiality**: Never share your PIN
- **Conscious Approval**: Carefully review each request before approving
- **Secure Environment**: Run the bunker in a trusted environment
- **Trusted Relays**: Only use trusted relays in the connection URI
- **Authenticity**: Verify device authenticity before use

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

--
