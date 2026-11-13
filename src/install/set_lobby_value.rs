//! One string value (the data and its NUL) set under the lobby's registry key with
//! `RegSetValueExA`, for `install::register_lobby`; returns the call's result.

use crate::errors::Refused;
use crate::install::read_registry::Registry;

const REG_SZ: u32 = 1;

/// The text `data` (its NUL added) as the value `name` of the lobby's key.
pub fn set_lobby_value(
    install: &crate::install::owner::Install,
    name: &[u8],
    data: &[u8],
    registry: &mut dyn Registry,
) -> Result<u32, Refused> {
    let mut text = data.to_vec();
    text.push(0);
    Ok(registry.set_value(install.run.reg_key.get(), name, REG_SZ, &text))
}
