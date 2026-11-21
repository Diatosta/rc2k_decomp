//! `menu::set_player_entry_kind`: the current player's (`event.current_player`) `entry`
//! 1 when the inspection score is 100 or more (signed), else 2.
//!
//! Refuses a current player past 3 or missing, before
//! anything.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn set_player_entry_kind(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    let n: u32 = screens.event.read().current_player;
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let score: i32 = screens.state.read().driver_stats.score as i32;
    player(menus, n).update(|q| q.entry = rc2k_core::menu::player::entry_kind(score));
    Ok(())
}
