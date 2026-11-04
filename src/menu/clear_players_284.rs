//! `menu::clear_players_284` (0x4A9679): every player's `hud_bits` set to -1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The four players of `g_players`. Called at the start of each kind of event (`sub_482382` and
//! seven more).

use crate::errors::Refused;
use crate::menu::players_store::PLAYER_TABLE;

pub fn clear_players_284() -> Result<(), Refused> {
    for player in &PLAYER_TABLE.players {
        player.update(|p| p.hud_bits = -1);
    }
    Ok(())
}
