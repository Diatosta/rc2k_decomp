//! `menu::set_player_entry_kind` (0x4A1922): the current player's (`dword_75598C`) +0xD0
//! 1 when `dword_75AC80` is 100 or more (signed), else 2.
//!
//! Refuses a current player past 3 or missing, before
//! anything.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Screens;
use crate::menu::standings::*;

/// The Rust version.
pub fn set_player_entry_kind(screens: &Screens) -> Result<(), Refused> {
    let n: u32 = screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::player::entry_kind`.
    let score: i32 = screens.state.driver_stats.score.get() as i32;
    player(n).update(|q| q.entry = rc2k_core::menu::player::entry_kind(score));
    Ok(())
}
