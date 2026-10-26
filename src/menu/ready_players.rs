//! `menu::ready_players` (0x471F7E): each counted player (0 .. `dword_755984`, the
//! original's `do .. while`) taking part again: +0xD0 1, +0x88 0, +0x94 0.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `saves::load_savegame`, the single rally's and the time trial's multi-player events.
//! Refuses (before anything) a player count of 0 or past 4 or a player missing .

use crate::errors::Refused;
use crate::menu::standings::{counted_players, put_all};

pub fn ready_players() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::event::PLAYER_READY`.
    for p in counted_players()? {
        put_all(p, &rc2k_core::menu::event::PLAYER_READY)?;
    }
    Ok(())
}
