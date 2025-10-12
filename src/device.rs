use anyhow::{Result, anyhow};
use ctap_hid_fido2::{
    FidoKeyHidFactory, LibCfg, fidokey::FidoKeyHid, fidokey::get_info::InfoOption,
    get_fidokey_devices,
};

pub fn find_fido_device() -> Result<FidoKeyHid> {
    let devices = get_fidokey_devices();
    if devices.is_empty() {
        return Err(anyhow!("No FIDO2 HID device connected."));
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
