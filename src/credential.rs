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
    println!("Configurando credencial...");
    let pin = get_pin_from_user()?;

    if let Ok(assertion) = device.get_assertion(RP_ID, CHALLENGE, &[], Some(pin.as_str())) {
        println!("Credencial encontrada.");
        return Ok(assertion.credential_id);
    }

    println!("Criando nova credencial...");
    let user = PublicKeyCredentialUserEntity {
        id: b"user-id-for-large-blob".to_vec(),
        name: "usuario.teste".to_string(),
        display_name: "Usuário de Teste".to_string(),
    };
    
    let hmac_extension = MakeExtension::HmacSecret(Some(true));
    let extensions = vec![hmac_extension];
    
    let args = MakeCredentialArgs {
        rpid: RP_ID.to_string(),
        challenge: CHALLENGE.to_vec(),
        pin: Some(pin.as_str()),
        key_types: vec![CredentialSupportedKeyType::Ecdsa256],
        uv: None,
        exclude_list: vec![],
        user_entity: Some(user),
        rk: Some(true),
        extensions: Some(extensions),
    };
    
    let attestation = device
        .make_credential_with_args(&args)
        .context("Falha ao criar a credencial.")?;
        
    println!("Credencial criada com sucesso!");
    Ok(attestation.credential_descriptor.id)
}

pub fn get_hmac_secret(device: &mut FidoKeyHid, credential_id: &[u8], salt: &[u8; 32]) -> Result<[u8; 32]> {
    let pin = get_pin_from_user()?;
    
    let hmac_extension = GetExtension::HmacSecret(Some(*salt));
    let extensions = vec![hmac_extension];
    
    let assertion = device
        .get_assertion_with_extensios(
            RP_ID,
            CHALLENGE,
            &[credential_id.to_vec()],
            Some(pin.as_str()),
            Some(&extensions),
        )
        .context("Falha ao obter chave de criptografia")?;
    
    for extension in &assertion.extensions {
        if let AssertionExtension::HmacSecret(Some(hmac_secret)) = extension {
            return Ok(*hmac_secret);
        }
    }
    
    Err(anyhow::anyhow!("Chave de criptografia não encontrada"))
}
