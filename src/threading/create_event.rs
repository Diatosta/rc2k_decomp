//! `threading::create_event` (0x41103C): an event without a name,
//! `CreateEventA(0, manual_reset, signalled, 0)`.
//!
//! Takes: manual reset (0 or 1), signalled at first (0 or 1)
//! Returns the handle, or none (the original's carry flag).
//! One caller, `net::set_up` (an auto-reset event, not signalled), which raises fatal error 0x80168501
//! on the carry. The Win32 call is made from Rust.

// ---- Rust version ----

/// The handle, 0 for none.
pub fn create_event(manual_reset: u32, signalled: u32) -> u32 {
    crate::sync::create_event(manual_reset != 0, signalled != 0)
}

/// The flags the original's compare of a handle with 1 gives: (carry, zero).
// Converted: `rc2k_core::threading::handle_flags`.
pub use rc2k_core::threading::handle_flags;
