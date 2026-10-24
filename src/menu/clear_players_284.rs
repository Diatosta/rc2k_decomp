//! `menu::clear_players_284` (0x4A9679): every player's `field_284` set to -1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The four players of `g_players`. Called at the start of each kind of event (`sub_482382` and
//! seven more).

use crate::errors::Refused;
use crate::menu::player::{PLAYERS, Player, player_at};

pub fn clear_players_284() -> Result<(), Refused> {
    for n in 0..PLAYERS {
        let p = player_at(n);
        if p == 0 {
            return Err(crate::errors::BadInput("a player missing from g_players").into());
        }
    }
    for n in 0..PLAYERS {
        unsafe { (*(player_at(n) as *mut Player)).field_284 = -1 };
    }
    Ok(())
}
