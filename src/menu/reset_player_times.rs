//! `menu::reset_player_times` (0x4813D6): the four players' +0x88 3600000 (an hour in
//! milliseconds: no time yet) and +0xD0 6; `dword_75592C` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126` (three times). Refuses a missing player, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn reset_player_times() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::Player::reset_times`.
    for n in 0..4 {
        player(n).update(rc2k_core::menu::player::Player::reset_times);
    }
    crate::menu::owner::menus()
        .screens
        .event
        .event_goes_on
        .set(0u32);
    Ok(())
}
