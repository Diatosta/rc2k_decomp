//! `install::unregister_lobby` (0x4190E9): the game's DirectPlay lobby key removed:
//! `RegDeleteKeyA(HKEY_LOCAL_MACHINE, "SOFTWARE\Microsoft\DirectPlay\Applications\RC")`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::uninstall`. The call through `api`. Shadow mode runs the original
//! alone. Self-test: a copy whose one call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{Api, REG_DELETE_KEY, a};
use crate::race::viewer::regs_entry;

const KEY_NAME: usize = 0x5AB419; // "SOFTWARE\Microsoft\DirectPlay\Applications\RC"
const HKLM: u32 = 0x8000_0002;

pub fn unregister_lobby(entry: &Regs, api: Api) -> Result<Regs, Refused> {
    api(REG_DELETE_KEY, &[HKLM, a(KEY_NAME)]);
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        unregister_lobby(r, &mut crate::install::installer::game_api)
    })
}
