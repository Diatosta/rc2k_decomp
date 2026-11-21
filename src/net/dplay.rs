//! DirectPlay's `CoCreateInstance` for the game's objects: the `IDirectPlay4A` and the
//! `IDirectPlayLobby2A` are `net::dp_objects`' ids, their methods its functions.

/// The session layer's object of `class` with `interface` (`net::dp_objects`), or the error.
pub(super) fn create_instance(class: &[u8; 16], interface: &[u8; 16]) -> Result<u32, u32> {
    crate::net::dp_objects::create(class, interface)
}
