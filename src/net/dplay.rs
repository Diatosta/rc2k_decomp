//! DirectPlay's `CoCreateInstance` for the game's objects: the `IDirectPlay4A` and the
//! `IDirectPlayLobby2A` are `net::dp_objects`' ids, their methods its functions.

/// `CoCreateInstance(class, 0, CLSCTX_INPROC_SERVER, interface, out)`; in the standalone
/// program DirectPlay's objects made in Rust (`net::dp_objects`): the object, or the error.
pub fn create_instance(class: &[u8; 16], interface: &[u8; 16]) -> Result<u32, u32> {
    crate::net::dp_objects::create(class, interface)
}
