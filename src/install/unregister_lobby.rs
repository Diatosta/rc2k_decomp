//! `install::unregister_lobby` (0x4190E9): the game's DirectPlay lobby key removed:
//! `RegDeleteKeyA(HKEY_LOCAL_MACHINE, "SOFTWARE\Microsoft\DirectPlay\Applications\RC")`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::uninstall`. The call through `api`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{Api, REG_DELETE_KEY, a};

const KEY_NAME: usize = 0x5AB419; // "SOFTWARE\Microsoft\DirectPlay\Applications\RC"
const HKLM: u32 = 0x8000_0002;

pub fn unregister_lobby(api: Api) {
    api(REG_DELETE_KEY, &[HKLM, a(KEY_NAME)]);
}
