//! `service::reset_tyre_wear` (0x486EDD): zeroes six dwords of the current car: +0x1F8 ..
//! +0x208 and +0x108 (the tyre damage shown in the service area).
//!
//! Three callers. The one in `service::apply_repairs` saves and restores +0x108 around the call, so
//! there only the five at +0x1F8 are cleared; what they hold has not been worked out, and "wear" is
//! a reading. Refuses a missing player.

use crate::errors::BadInput;

// ---- game data ----

// five dwords
// ---- Rust version ----

pub fn reset_tyre_wear() -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::car` (`Player::reset_tyre_wear`).
    crate::service::repair_parts::with_player(rc2k_core::menu::player::Player::reset_tyre_wear)
}
