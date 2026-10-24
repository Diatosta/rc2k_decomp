//! `install::set_lobby_value` (0x4190D0): one string value set under the lobby's registry
//! key (`register_lobby`): its data's length found (`strings::text_length`) and one more
//! (the NUL), and `RegSetValueExA(phkResult, name, 0, REG_SZ, data, len + 1)`.
//!
//! In:  eax the value's name, ebx the data (NUL-terminated)
//! Out: eax `RegSetValueExA`'s result; ebx, edx kept, ecx the length plus one
//! Caller `install::register_lobby` (five times). The call through `api`
//! (`installer::game_api`: `RegSetValueExA`). Shadow mode runs the original alone.
//! Self-test: a copy whose two calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{Api, REG_KEY, REG_SET_VALUE, a, rd};

const REG_SZ: u32 = 1;

/// The value `name` (eax) set to the text `data` (ebx); `RegSetValueExA`'s answer (no
/// caller reads it).
pub fn set_lobby_value(name: u32, data: u32, api: Api) -> Result<u32, Refused> {
    let len = unsafe { crate::strings::text_length::text_length(data as *const u8) }?;
    let key = rd(a(REG_KEY));
    Ok(api(
        REG_SET_VALUE,
        &[key, name, 0, REG_SZ, data, len.wrapping_add(1)],
    ))
}
