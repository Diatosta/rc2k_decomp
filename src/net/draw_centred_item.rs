//! `net::draw_centred_item` (0x416B53): an item's string drawn (`text::draw_string_by_id`)
//! at x ecx, y ebp, in `unk_51E744`, or `stru_51E684` when it is the highlighted item and
//! edited or in the blink's lit half (as `net::draw_button`); ebp moves down by the font's
//! line height (+8).
//!
//! In:  eax = the string, ecx = x, esi = the item, ebp = y
//! Out: ebx = the font, edx = the y, ebp lower; eax .. edx as the drawing leaves them; no
//!      `pusha`
//! Caller: `net::settings_frame` (its last item). Its call through `part`. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose call goes to a stand-in, against
//! the Rust version: highlighted or not, edited, blinking.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::draw_button::item_font;
use crate::net::lobby::{DRAW_STRING, Part, lobby_entry};

const PLAIN: usize = 0x51E744; // unk_51E744
const LIT: usize = 0x51E684; // stru_51E684

/// The Rust version: the registers it leaves.
pub fn draw_centred_item(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::screens::item_lit` (in `item_font`).
    r.ebx = item_font(r.esi, PLAIN, LIT);
    r.edx = r.ebp;
    r.ebp = r
        .ebp
        .wrapping_add(unsafe { ((r.ebx + 8) as *const u32).read_unaligned() });
    part(DRAW_STRING, &mut r)?;
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, draw_centred_item)
}
