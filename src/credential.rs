use anyhow::{Context, Result};
use ctap_hid_fido2::{
    public_key_credential_user_entity::PublicKeyCredentialUserEntity,
    fidokey::FidoKeyHid,
};
use crate::auth::get_pin_from_user;

const RP_ID: &str = "example.com";
const CHALLENGE: &[u8] = b"a-random-challenge-string";

pub fn get_credential_id(device: &mut FidoKeyHid) -> Result<Vec<u8>> {
    println!("Verificando ou criando credencial residente...");
    let pin = get_pin_from_user()?;

    if let Ok(assertion) = device.get_assertion(RP_ID, CHALLENGE, &[], Some(pin.as_str())) {
        println!("Credencial residente encontrada.");
        return Ok(assertion.credential_id);
    }

    println!("Nenhuma credencial encontrada. Criando uma nova...");
    let user = PublicKeyCredentialUserEntity {
        id: b"user-id-for-large-blob".to_vec(),
        name: "usuario.teste".to_string(),
        display_name: "Usuário de Teste".to_string(),
    };
    
    let attestation = device
        .make_credential_rk(
            RP_ID,
            CHALLENGE,
            Some(pin.as_str()),
            &user,
        )
        .context("Falha ao criar a credencial residente.")?;
        
    println!("Credencial residente criada com sucesso!");
    Ok(attestation.credential_descriptor.id)
}
