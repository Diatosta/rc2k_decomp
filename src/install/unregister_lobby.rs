//! `install::unregister_lobby` (0x4190E9): the game's DirectPlay lobby key removed:
//! `RegDeleteKeyA(HKEY_LOCAL_MACHINE, "SOFTWARE\Microsoft\DirectPlay\Applications\RC")`.
//!
//! Caller `install::uninstall`. The call through `api`.

use crate::install::read_registry::Registry;

pub fn unregister_lobby(install: &crate::install::owner::Install, registry: &mut dyn Registry) {
    registry.delete(crate::install::store::text_of(
        &install.lobby_keys.registry_key,
    ));
}
