use crate::auth::get_pin_from_user;
use anyhow::{Context, Result};
use ctap_hid_fido2::{
    fidokey::FidoKeyHid,
    fidokey::get_assertion::get_assertion_params::{
        Extension as GetExtension, Extension as AssertionExtension,
    },
    fidokey::make_credential::make_credential_params::{
        CredentialSupportedKeyType, Extension as MakeExtension, MakeCredentialArgs,
    },
    public_key_credential_user_entity::PublicKeyCredentialUserEntity,
};
use zeroize::Zeroize;

const RP_ID: &str = "nostr.bunker.yubikey";
const CHALLENGE: &[u8] = b"yubikey-nostr-bunker-challenge";


pub fn get_credential_id(device: &mut FidoKeyHid) -> Result<Vec<u8>> {
    let mut pin = get_pin_from_user()?;

    if let Ok(assertion) = device.get_assertion(RP_ID, CHALLENGE, &[], Some(pin.as_str())) {
        pin.zeroize();

        return Ok(assertion.credential_id);
    }

    let user = PublicKeyCredentialUserEntity {
        id: b"user-id-for-large-blob".to_vec(),
        name: "test.user".to_string(),
        display_name: "Test User".to_string(),
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
        .context("Failed to create credential.")?;

    pin.zeroize();

    Ok(attestation.credential_descriptor.id)
}

pub fn get_hmac_secret(
    device: &mut FidoKeyHid,
    credential_id: &[u8],
    salt: &[u8; 32],
) -> Result<[u8; 32]> {
    let mut pin = get_pin_from_user()?;

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
        .context("Failed to get encryption key")?;

    pin.zeroize();

    for extension in &assertion.extensions {
        if let AssertionExtension::HmacSecret(Some(hmac_secret)) = extension {
            return Ok(*hmac_secret);
        }
    }

    Err(anyhow::anyhow!("Encryption key not found"))
}
