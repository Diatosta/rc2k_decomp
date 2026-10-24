//! `net::lobby_button` (0x41802F): one of the lobby's buttons, right to left: drawn
//! (`net::draw_button`, flags 1, x edi, y ebp), dim (flag 2) while this player is ready (state & 1)
//! for all but item 3, and for item 0 after a race (`dword_52AD80` 1); edi then
//! moves left past it (its width `dword_5AB4A0` + 0x44, and 8 more). (The code after its
//! `ret`, 0x41806E .. 0x418082, is reached by nothing.)
//!
//! In:  the string, the item, x, y
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
use crate::net::draw_button::draw_button;
use crate::net::lobby::{BACK_FROM_RACE, OWN, g, gb};
use rc2k_core::net::screens;

const WIDTH: usize = 0x5AB4A0; // dword_5AB4A0

/// The Rust version: the button `string` for item `item` at (`x`, `y`); returns the next
/// button's x (the original's edi; its ebx, ecx and edx are read by no caller:
/// `net::lobby_frame`).
pub fn lobby_button(string: u32, item: u32, x: u32, y: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens` (`lobby_button_flags`, `next_button_x`).
    let flags = screens::lobby_button_flags(gb(OWN), item, g(BACK_FROM_RACE));
    draw_button(string, flags, x, y, item)?;
    Ok(screens::next_button_x(x, g(WIDTH)))
}
