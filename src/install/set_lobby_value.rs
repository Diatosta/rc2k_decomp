//! `install::set_lobby_value` (0x4190D0): one string value set under the lobby's registry
//! key (`register_lobby`): its data's length found (`strings::text_length`) and one more
//! (the NUL), and `RegSetValueExA(phkResult, name, 0, REG_SZ, data, len + 1)`.
//!
//! In:  eax the value's name, ebx the data (NUL-terminated)
//! Out: eax `RegSetValueExA`'s result; ebx, edx kept, ecx the length plus one
//! Caller `install::register_lobby` (five times). The call through `api`
//! (`installer::game_api`: `RegSetValueExA`).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::read_registry::Registry;
use crate::install::vars::INSTALLER_RUN;

const REG_SZ: u32 = 1;

/// The Rust version: the text `data` (its NUL added) as the value `name` of the lobby's key.
pub fn set_lobby_value(
    name: &[u8],
    data: &[u8],
    registry: &mut dyn Registry,
) -> Result<u32, Refused> {
    let mut text = data.to_vec();
    text.push(0);
    Ok(registry.set_value(INSTALLER_RUN.reg_key.get(), name, REG_SZ, &text))
}
