use anyhow::{anyhow, Result};
use ctap_hid_fido2::{
    get_fidokey_devices, FidoKeyHidFactory, LibCfg,
    fidokey::FidoKeyHid,
    fidokey::get_info::InfoOption,
};

pub fn find_fido_device() -> Result<FidoKeyHid> {
    let devices = get_fidokey_devices();
    if devices.is_empty() {
        return Err(anyhow!("Nenhum dispositivo FIDO2 HID conectado."));
    }
    let cfg = LibCfg::init();
    FidoKeyHidFactory::create(&cfg)
}

pub fn is_supported(device: &FidoKeyHid) -> Result<bool> {
    if device
        .enable_info_option(&InfoOption::LargeBlobs)?
        .is_some()
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
