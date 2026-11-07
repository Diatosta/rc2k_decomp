//! `install::unregister_lobby` (0x4190E9): the game's DirectPlay lobby key removed:
//! `RegDeleteKeyA(HKEY_LOCAL_MACHINE, "SOFTWARE\Microsoft\DirectPlay\Applications\RC")`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::uninstall`. The call through `api`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::read_registry::Registry;

pub fn unregister_lobby(
    install: &'static crate::install::owner::Install,
    registry: &mut dyn Registry,
) {
    registry.delete(crate::install::store::text_of(
        &install.lobby_keys.registry_key,
    ));
}
