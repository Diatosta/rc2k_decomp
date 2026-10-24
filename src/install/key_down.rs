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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use rc2k_core::install::items::ItemKey;

/// Whether the key was taken (the original's zero).
pub fn key_down(item: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    item_key(item, ItemKey::Down, 0)
}
