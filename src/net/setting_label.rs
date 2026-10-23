//! `net::setting_label` (0x416AF8): a setting's label as a button (`net::draw_button`, flags
//! 0, x 0x190, y ebp), and ebp 0x20 lower. (The code after its `ret`, 0x416B0D .. 0x416B52,
//! is reached by nothing.)
//!
//! In:  eax = the string, esi = the item, ebp = y
//! Out: ebx = 0, ecx = 0x190, edx = the y, ebp + 0x20; the rest as the call leaves it (it
//!      keeps every register); no `pusha`
//! Caller: `net::settings_frame` (four times). Its call through `part`. Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose call goes to a stand-in, against the
//! Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, lobby_entry};

const DRAW_BUTTON: u32 = 0x416CE0; // net::draw_button

/// The Rust version: the registers it leaves.
pub fn setting_label(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.ebx = 0;
    r.ecx = 0x190;
    r.edx = r.ebp;
    part(DRAW_BUTTON, &mut r)?;
    r.ebp = r.ebp.wrapping_add(0x20);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, setting_label)
}
