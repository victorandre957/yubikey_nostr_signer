use anyhow::Result;
use std::io::{self, Write};

pub fn get_pin_from_user() -> Result<String> {
    print!("Por favor, insira o PIN da sua chave de segurança: ");
    io::stdout().flush()?;
    let pin = rpassword::read_password()?;
    Ok(pin)
}
