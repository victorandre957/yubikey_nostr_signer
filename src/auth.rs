use anyhow::Result;
use std::io::{self, Write};

pub fn get_pin_from_user() -> Result<String> {
    print!("Enter your PIN: ");
    io::stdout().flush()?;
    let pin = rpassword::read_password()?;
    Ok(pin)
}
