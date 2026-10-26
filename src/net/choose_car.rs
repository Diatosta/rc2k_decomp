//! `net::choose_car` (0x4182FA): the lobby's car item: `dword_75598C` 0, the car screen
//! (`menu::select_car_loop` on `net::car_list`'s list, from player 1's car, the Escape flag
//! `dword_755A3C` 0 first); unless it was left with Escape, the car taken
//! (`menu::take_shown_car`) and this player's record given it (+1 the car, +2 its colours);
//! then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::lobby_loop`. Its calls through `part` (the car screen through its hook).
//! The screen runs inside it: ungated. Shadow mode runs the original alone. Self-test: a copy
//! whose calls go to stand-ins, against the Rust version, left with Escape or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{OWN, Part, SELECT_CAR, TAKE_SHOWN_CAR, g, lobby_entry, p, pb};

const CAR_LIST: u32 = 0x4AAAF7; // net::car_list
const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background
const CLEARED: usize = 0x75598C; // dword_75598C
pub const ESCAPED: usize = 0x755A3C; // dword_755A3C: the screen was left with Escape
const CAR: usize = 0x755A54; // g_player1.car_number
const COLOURS: usize = 0x755A7C; // g_player1.field_30

/// The Rust version: the registers it leaves (the entry's).
pub fn choose_car(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(CLEARED, 0);
    part(CAR_LIST, &mut r)?;
    p(ESCAPED, 0);
    r.eax = g(CAR);
    part(SELECT_CAR, &mut r)?;
    if g(ESCAPED) != 1 {
        part(TAKE_SHOWN_CAR, &mut r)?;
        r.eax = g(CAR);
        pb(OWN + 1, r.eax as u8);
        r.eax = g(COLOURS);
        pb(OWN + 2, r.eax as u8);
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
    lobby_entry(r, choose_car)
}
