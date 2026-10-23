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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{Api, REG_KEY, REG_SET_VALUE, a, rd};
use crate::race::viewer::regs_entry;

const REG_SZ: u32 = 1;

pub fn set_lobby_value(entry: &Regs, api: Api) -> Result<Regs, Refused> {
    let mut r = *entry;
    let len = unsafe { crate::strings::text_length::text_length(r.ebx as *const u8) }?;
    r.ecx = len.wrapping_add(1);
    let key = rd(a(REG_KEY));
    r.eax = api(REG_SET_VALUE, &[key, r.eax, 0, REG_SZ, r.ebx, r.ecx]);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        set_lobby_value(r, &mut crate::install::installer::game_api)
    })
}
