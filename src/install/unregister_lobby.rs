//! The game's DirectPlay lobby key removed.

use crate::install::read_registry::Registry;

pub(super) fn unregister_lobby(
    install: &crate::install::owner::Install,
    registry: &mut dyn Registry,
) {
    registry.delete(crate::install::store::text_of(
        &install.lobby_keys.registry_key,
    ));
}
