//! `net::choose_stage` (0x418347): the lobby's stage item (the host's): the stage screen
//! (`menu::select_stage_loop`, the Escape flag `dword_755A3C` 0 first); unless it was left
//! with Escape, this player's record given the stage (+4 from `dword_75ACD4`) and its
//! direction (+5 from `dword_75ACF0`); then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::lobby_loop`. Its calls through `part` (the stage screen through its hook).
//! The screen runs inside it: ungated. Shadow mode runs the original alone. Self-test: a copy
//! whose calls go to stand-ins, against the Rust version, left with Escape or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::choose_car::ESCAPED;
use crate::net::fill_own_player::{DIRECTION, STAGE};
use crate::net::lobby::{OWN, Part, SELECT_STAGE, g, lobby_entry, p, pb};

const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background

/// The Rust version: the registers it leaves (the entry's).
pub fn choose_stage(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(ESCAPED, 0);
    part(SELECT_STAGE, &mut r)?;
    if g(ESCAPED) != 1 {
        r.eax = g(STAGE);
        pb(OWN + 4, r.eax as u8);
        r.eax = g(DIRECTION);
        pb(OWN + 5, r.eax as u8);
    }
    part(SHOW_BACKGROUND, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, choose_stage)
}
