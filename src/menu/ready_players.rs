//! `menu::ready_players` (0x471F7E): each counted player (0 .. `dword_755984`, the
//! original's `do .. while`) taking part again: +0xD0 1, +0x88 0, +0x94 0.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `saves::load_savegame`, the single rally's and the time trial's multi-player
//! events. Refuses (before anything) a player count of 0 or past 4 or a player missing
//! (the original runs on past the table or reads through 0). Self-test: the game's players
//! with test values, for each count, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::{counted_players, put_all};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

pub fn ready_players() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::event::PLAYER_READY`.
    for p in counted_players()? {
        put_all(p, &rc2k_core::menu::event::PLAYER_READY);
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    ready_players()?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
