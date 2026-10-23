//! `net::lobby_button` (0x41802F): one of the lobby's buttons, right to left: drawn
//! (`net::draw_button`, flags 1, x edi, y ebp), dim (flag 2) while this player is ready (state & 1)
//! for all but item 3, and for item 0 after a race (`dword_52AD80` 1); edi then
//! moves left past it (its width `dword_5AB4A0` + 0x44, and 8 more). (The code after its
//! `ret`, 0x41806E .. 0x418082, is reached by nothing.)
//!
//! In:  eax = the string, esi = the item, edi = x, ebp = y
//! Out: ebx = the flags, ecx = the width + 0x44, edx = y, edi moved; the rest kept; no
//!      `pusha`
//! Caller: `net::lobby_frame` (four times). Its call through `part`. Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose call goes to a stand-in, against the
//! Rust version, on each item, ready or not, after a race or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{BACK_FROM_RACE, OWN, Part, g, gb, lobby_entry};
use rc2k_core::net::screens;

const DRAW_BUTTON: u32 = 0x416CE0; // net::draw_button
const WIDTH: usize = 0x5AB4A0; // dword_5AB4A0

/// The Rust version: the registers it leaves.
pub fn lobby_button(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::screens` (`lobby_button_flags`, `next_button_x`).
    r.ebx = screens::lobby_button_flags(gb(OWN), r.esi, g(BACK_FROM_RACE));
    r.ecx = r.edi;
    r.edx = r.ebp;
    part(DRAW_BUTTON, &mut r)?;
    let width = g(WIDTH);
    r.ecx = width.wrapping_add(screens::FRAME_MORE);
    r.edi = screens::next_button_x(r.edi, width);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, lobby_button)
}
