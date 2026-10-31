//! `service::reset_tyre_wear` (0x486EDD): zeroes six dwords of the current car: +0x1F8 ..
//! +0x208 and +0x108 (the tyre damage shown in the service area).
//!
//! In:  nothing (eax is saved and restored)
//! Out: every register preserved
//! Three callers. The one in `sub_46C14D` saves and restores +0x108 around the call, so there
//! only the five at +0x1F8 are cleared; what they hold has not been worked out, and "wear" is
//! a reading. Refuses a missing player.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;

// ---- game data ----

// five dwords
// ---- Rust version ----

pub fn reset_tyre_wear() -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::car` (`Player::reset_tyre_wear`).
    crate::service::repair_parts::with_player(rc2k_core::menu::player::Player::reset_tyre_wear)
}
