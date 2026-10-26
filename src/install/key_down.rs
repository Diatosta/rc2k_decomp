//! `install::key_down` (0x414794): key 0x6B on an item: for choices shown upright (kind
//! 0x22 of `[esi] & 0x2F`) the choice after (`install::choice_on`).
//!
//! In:  esi the item
//! Out: bl the kind (`[esi] & 0x2F`); with choices as `choice_on` leaves them (zero set),
//!      else the flags of `cmp bl, 22h` (zero clear)
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::key_up::item_key;
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::ItemKey;

pub fn key_down(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    item_key(entry, ItemKey::Down)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &key_down)
}
