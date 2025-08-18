use anyhow::{Context, Result};
use ctap_hid_fido2::{
    public_key_credential_user_entity::PublicKeyCredentialUserEntity,
    fidokey::FidoKeyHid,
    fidokey::make_credential::make_credential_params::{
        Extension as MakeExtension,
        MakeCredentialArgs,
        CredentialSupportedKeyType,
    },
    fidokey::get_assertion::get_assertion_params::{Extension as GetExtension, Extension as AssertionExtension},
};
use crate::auth::get_pin_from_user;

const RP_ID: &str = "example.com";
const CHALLENGE: &[u8] = b"a-random-challenge-string";

pub fn get_credential_id(device: &mut FidoKeyHid) -> Result<Vec<u8>> {
    println!("Verificando ou criando credencial residente com HMAC-secret...");
    let pin = get_pin_from_user()?;

    if let Ok(assertion) = device.get_assertion(RP_ID, CHALLENGE, &[], Some(pin.as_str())) {
        println!("Credencial residente encontrada.");
        return Ok(assertion.credential_id);
    }

    println!("Nenhuma credencial encontrada. Criando uma nova com suporte a HMAC-secret...");
    let user = PublicKeyCredentialUserEntity {
        id: b"user-id-for-large-blob".to_vec(),
        name: "usuario.teste".to_string(),
        display_name: "Usuário de Teste".to_string(),
    };
    
    // Create HMAC-secret extension for make_credential
    let hmac_extension = MakeExtension::HmacSecret(Some(true));
    let extensions = vec![hmac_extension];
    
    // Create args with all necessary parameters for resident key + HMAC-secret
    let args = MakeCredentialArgs {
        rpid: RP_ID.to_string(),
        challenge: CHALLENGE.to_vec(),
        pin: Some(pin.as_str()),
        key_types: vec![CredentialSupportedKeyType::Ecdsa256],
        uv: None,
        exclude_list: vec![],
        user_entity: Some(user),
        rk: Some(true), // Enable resident key
        extensions: Some(extensions),
    };
    
    let attestation = device
        .make_credential_with_args(&args)
        .context("Falha ao criar a credencial residente com HMAC-secret.")?;
        
    println!("Credencial residente criada com sucesso (com suporte a HMAC-secret)!");
    Ok(attestation.credential_descriptor.id)
}

/// Get HMAC secret using FIDO2 HMAC-secret extension
/// This function requests a 32-byte HMAC secret from the authenticator using a salt
pub fn get_hmac_secret(device: &mut FidoKeyHid, credential_id: &[u8], salt: &[u8; 32]) -> Result<[u8; 32]> {
    let pin = get_pin_from_user()?;
    
    // Create HMAC-secret extension for get_assertion
    let hmac_extension = GetExtension::HmacSecret(Some(*salt));
    let extensions = vec![hmac_extension];
    
    // Get assertion with HMAC-secret extension
    let assertion = device
        .get_assertion_with_extensios(
            RP_ID,
            CHALLENGE,
            &[credential_id.to_vec()],
            Some(pin.as_str()),
            Some(&extensions),
        )
        .context("Falha ao obter secret HMAC do authenticator")?;
    
    // Extract HMAC secret from assertion extensions
    for extension in &assertion.extensions {
        if let AssertionExtension::HmacSecret(Some(hmac_secret)) = extension {
            return Ok(*hmac_secret);
        }
    }
    
    Err(anyhow::anyhow!("HMAC secret não encontrado na resposta do authenticator"))
}
