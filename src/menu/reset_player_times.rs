//! `menu::reset_player_times` (0x4813D6): the four players' +0x88 3600000 (an hour in
//! milliseconds: no time yet) and +0xD0 6; `dword_75592C` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126` (three times). Refuses a missing player, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn reset_player_times() -> Result<(), Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    // Converted: `rc2k_core::menu::event::PLAYER_TIMES`.
    for n in 0..4 {
        put_all(player(n), &rc2k_core::menu::event::PLAYER_TIMES)?;
    }
    EVENT_STATE.event_goes_on.set(0u32);
    Ok(())
}
