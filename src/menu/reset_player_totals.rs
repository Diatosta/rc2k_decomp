//! `menu::reset_player_totals` (0x471FB8): the counted players' (`dword_755984`) +0x8C and
//! +0x88 3600000.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_471F0D`. Refuses a player count of 0 or past 4 or a missing player, before
//! anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn reset_player_totals() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::Player::reset_totals`.
    for p in counted_players()? {
        p.update(rc2k_core::menu::player::Player::reset_totals);
    }
    Ok(())
}
