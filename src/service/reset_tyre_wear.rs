//! Zeroes six dwords of the current car: five of tyre wear and the tyre damage shown in the
//! service area.
//!
//! `service::apply_repairs` saves and restores the damage around the call, so there only the
//! five are cleared; what they hold has not been worked out, and "wear" is a reading. Refuses a
//! missing player.

use crate::errors::BadInput;
use crate::menu::owner::Menus;

pub fn reset_tyre_wear(menus: &Menus) -> Result<(), BadInput> {
    crate::service::repair_parts::with_player(
        menus,
        rc2k_core::menu::player::Player::reset_tyre_wear,
    )
}
