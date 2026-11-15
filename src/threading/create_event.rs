//! An event without a name, `CreateEventA(0, manual_reset, signalled, 0)`. The caller
//! (`net::set_up`, an auto-reset event, not signalled) raises fatal error 0x80168501 on none.

/// The handle, 0 for none.
pub fn create_event(manual_reset: u32, signalled: u32) -> u32 {
    crate::platform::sync::create_event(manual_reset != 0, signalled != 0)
}

/// The (carry, zero) flags a handle compared with 1 gives.
pub use rc2k_core::threading::handle_flags;
