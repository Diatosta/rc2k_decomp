//! `net::begin_edit` (0x416E1A): an item's text starts being edited: the item (`dword_52A4B8`),
//! the text (`dword_52A4B4`), the widest it may be (`dword_52A4BC`) and the font it is
//! measured with (`dword_52A4C0`) noted, and the text's 64 bytes kept (`byte_95EAF8`) for
//! Escape.
//!
//! In:  eax = the item, ebx = the text, ecx = the width, edx = the font
//! Out: every register kept (`pusha` .. `popa`)
//! Callers: `net::begin_setting_edit`, `net::begin_chat`. Shadow mode: both run, the five
//! variables compared. Self-test: a copy against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    EDIT_FONT, EDIT_SAVED, EDIT_TEXT, EDIT_WIDTH, EDITED, Part, a, movsb, p, region,
};

/// The Rust version: the registers it leaves (the entry's).
pub fn begin_edit(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    p(EDITED, entry.eax);
    p(EDIT_TEXT, entry.ebx);
    p(EDIT_WIDTH, entry.ecx);
    p(EDIT_FONT, entry.edx);
    movsb(a(EDIT_SAVED), entry.ebx, 0x40);
    Ok(*entry)
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    vec![region(EDIT_TEXT, 0x10), region(EDIT_SAVED, 0x40)]
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| begin_edit(r, &mut crate::net::lobby::game_part))
}
