//! `memory::take_from_list` (0x402956): hands out the next bytes of a list's newest block.
//!
//! In:  eax = bytes, ebx = the list (`MemoryList`, after `memory::rewind_list`)
//! Out: eax = their address, carry clear; or eax = 0, carry set, if the list has fewer bytes
//!      left; everything else preserved
//! The list's cursor moves on and what remains is counted down. 12 call sites: the lexicon
//! compiler and the sprite and font builder cut their work memory this way.
//! Shadow mode runs the original alone (the cursor cannot move twice) and checks what it
//! left. Manual patch with a private copy (0x15 bytes, nothing to relocate): a conditional
//! jump sits in the first bytes. The name is a reading.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::memory::MemoryList;
use crate::pointers::{load_value, store_value};

// ---- Rust version ----

/// The address of the bytes, or None if the list has not that many left.
pub fn take_from_list(
    list: &dyn crate::pointers::Store,
    bytes: u32,
) -> Result<Option<u32>, BadInput> {
    let mut l = load_value::<MemoryList>(list, 0)?;
    let taken = l.take(bytes);
    store_value(list, 0, l)?;
    Ok(taken)
}

pub fn rewind_list(list: &dyn crate::pointers::Store) -> Result<(), BadInput> {
    let mut l = load_value::<MemoryList>(list, 0)?;
    l.rewind();
    store_value(list, 0, l)
}
