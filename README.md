## 📁 File Structure

```
yubikey_fido2_teste/
├── src/
│   ├── main.rs           # CLI interface and main application loop
│   ├── credential.rs     # FIDO2 credential management
│   ├── blob_operations.rs # Encryption, storage, and blob operations
│   ├── auth.rs          # PIN authentication utilities
│   ├── device.rs        # Device detection and initialization
│   └── lib.rs           # Library exports and common types
├── Cargo.toml           # Dependencies and project configuration
└── README.md           # This documentation
```

## 🔧 Module Details

### `main.rs` - Application Entry Point
- **Purpose**: Provides the command-line interface and main application loop
- **Key Functions**:
  - Device initialization and credential setup
  - Interactive menu system for user operations
  - Error handling and user feedback
- **Dependencies**: All other modules

### `credential.rs` - FIDO2 Protocol Management
- **Purpose**: Handles FIDO2 credential creation and HMAC-secret derivation
- **Key Functions**:
  - `get_credential_id()`: Creates resident credentials with HMAC-secret extension
  - `get_hmac_secret()`: Derives encryption keys from FIDO2 device
- **FIDO2 Features Used**:
  - Resident keys for persistent storage
  - HMAC-secret extension for key derivation
  - Proper extension handling and assertions

### `blob_operations.rs` - Encryption & Storage
- **Purpose**: Manages encrypted data storage, retrieval, and manipulation
- **Key Functions**:
  - `encrypt_data()`: AES-256-CBC encryption with random IV
  - `decrypt_data()`: Secure decryption with HMAC-derived keys
  - `write_blob()`: Store encrypted entries with ID support
  - `read_blob()`: Selective decryption of specific entries
  - `delete_single_entry()`: Secure deletion without decryption
- **Data Format**: `id:encrypted_data` with backward compatibility

### `auth.rs` - Authentication Utilities
- **Purpose**: Handles PIN authentication for FIDO2 operations
- **Key Functions**:
  - `get_pin()`: Secure PIN input with hidden characters
- **Security**: Uses `rpassword` for secure PIN entry

### `device.rs` - Device Management
- **Purpose**: FIDO2 device detection and initialization
- **Key Functions**:
  - Device enumeration and connection
  - Hardware compatibility validation

## 🚀 Getting Started

### Prerequisites

- **Hardware**: FIDO2-compatible device (YubiKey 5 series recommended)
- **Software**: Rust 1.70+ with Cargo

### Installation

1. **Clone the repository**:

2. **Build the application**:
   ```bash
   cargo build --release
   ```

3. **Run the application**:
   ```bash
   cargo run
   ```

### First Run

1. Connect your FIDO2 device
2. The application will automatically detect and initialize your device
3. Enter your device PIN when prompted
4. The system will create a credential if none exists
5. You're ready to start encrypting data!

## 💡 Usage Examples

### Storing Encrypted Data
```
Select an option:
1. Store encrypted data
2. Read encrypted data
3. Delete encrypted data
4. Exit

Choice: 1
Enter an ID for this entry: my-secret-key
Enter data to encrypt: super-secret-nostr-key
✓ Data encrypted and stored successfully!
```

### Reading Specific Data
```
Choice: 2

Existing entries:
1: my-secret-key
2: backup-key
3: master-key

Enter the number of the entry to decrypt (or 0 to cancel): 1
Enter your PIN: ****
Decrypted data: super-secret-nostr-key
```

### Secure Deletion
```
Choice: 3

Existing entries:
1: my-secret-key
2: backup-key
3: master-key

Enter the number of the entry to delete (or 0 to cancel): 2
✓ Entry deleted successfully!
```

## 🔒 Security Features

### Hardware-Backed Security
- **FIDO2 HMAC-secret**: Encryption keys never leave the hardware device
- **PIN Protection**: All operations require device PIN authentication
- **Resident Keys**: Credentials are stored securely on the device

### Encryption Standards
- **AES-256-CBC**: Industry-standard symmetric encryption
- **Random IV**: Each encryption uses a unique initialization vector
- **Salt-based Key Derivation**: HMAC-secret uses random salt for each key derivation

### Data Protection
- **Memory Safety**: Rust's ownership system prevents buffer overflows
- **No Key Storage**: Encryption keys are derived on-demand and never persisted
- **Secure Deletion**: Original data is overwritten in memory


## 🛠️ Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ctap-hid-fido2` | 3.5.5 | FIDO2 protocol implementation |
| `aes` | 0.8.4 | AES encryption |
| `cbc` | 0.1.2 | CBC mode for AES |
| `hex` | 0.4.3 | Hexadecimal encoding/decoding |
| `rand` | 0.8.5 | Cryptographic random number generation |
| `anyhow` | 1.0.86 | Error handling |
| `rpassword` | 7.3.1 | Secure password input |

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/new-feature`
3. Commit your changes: `git commit -am 'Add new feature'`
4. Push to the branch: `git push origin feat/new-feature`
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Security Considerations

- Always use a PIN-protected FIDO2 device
- Keep your device firmware updated
- Backup your encrypted data externally
- Never share your device PIN
- Verify device authenticity before use

## 🔗 References

- [FIDO2 Specification](https://fidoalliance.org/specs/fido-v2.1-ps-20210615/fido-client-to-authenticator-protocol-v2.1-ps-errata-20220621.html)
- [WebAuthn HMAC-secret Extension](https://w3c.github.io/webauthn/#sctn-hmac-secret-extension)
- [YubiKey FIDO2 Developer Guide](https://developers.yubico.com/FIDO2/)
- [ctap-hid-fido2 Documentation](https://docs.rs/ctap-hid-fido2/)

---

**Made with 🔐 for secure key management**
