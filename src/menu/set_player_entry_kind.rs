//! `menu::set_player_entry_kind` (0x4A1922): the current player's (`dword_75598C`) +0xD0
//! 1 when `dword_75AC80` is 100 or more (signed), else 2.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`. Refuses a current player past 3 or missing, before
//! anything.

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;

/// The Rust version (see the header): the registers it leaves.
pub fn set_player_entry_kind() -> Result<(), Refused> {
    let n: u32 = EVENT_STATE.current_player.get();
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::player::entry_kind`.
    let score: i32 = MENU_STATE.driver_stats.score.get() as i32;
    put(player(n) + 0xD0, rc2k_core::menu::player::entry_kind(score))?;
    Ok(())
}
