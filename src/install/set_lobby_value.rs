//! `install::set_lobby_value` (0x4190D0): one string value set under the lobby's registry
//! key (`register_lobby`): its data's length found (`strings::text_length`) and one more
//! (the NUL), and `RegSetValueExA(phkResult, name, 0, REG_SZ, data, len + 1)`.
//!
//! Takes the value's name and the data (NUL-terminated). Returns `RegSetValueExA`'s result.
//! Caller `install::register_lobby` (five times). The call through `api`
//! (`installer::game_api`: `RegSetValueExA`).

use crate::errors::Refused;
use crate::install::read_registry::Registry;

const REG_SZ: u32 = 1;

/// The text `data` (its NUL added) as the value `name` of the lobby's key.
pub fn set_lobby_value(
    install: &'static crate::install::owner::Install,
    name: &[u8],
    data: &[u8],
    registry: &mut dyn Registry,
) -> Result<u32, Refused> {
    let mut text = data.to_vec();
    text.push(0);
    Ok(registry.set_value(install.run.reg_key.get(), name, REG_SZ, &text))
}
