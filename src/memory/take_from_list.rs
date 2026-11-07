//! `memory::take_from_list` (0x402956): hands out the next bytes of a list's newest block.
//!
//! In:  eax = bytes, ebx = the list (`MemoryList`, after `memory::rewind_list`)
//! Out: eax = their address, carry clear; or eax = 0, carry set, if the list has fewer bytes
//!      left; everything else preserved
//! The list's cursor moves on and what remains is counted down. 12 call sites: the lexicon
//! compiler and the sprite and font builder cut their work memory this way.
//! Shadow mode runs the original alone (the cursor cannot move twice) and checks what it
//! left. Manual patch with a private copy (0x15 bytes, nothing to relocate): a conditional
//! jump sits in the first bytes. Refuses a null list. The name is a reading.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::memory::MemoryList;

// ---- Rust version ----

/// The address of the bytes, or None if the list has not that many left.
pub fn take_from_list(list: crate::pointers::GamePtr, bytes: u32) -> Result<Option<u32>, BadInput> {
    if list.is_null() {
        return Err(BadInput("no list to take from"));
    }
    let at = list;
    let mut l = at.read::<MemoryList>("a memory list")?;
    let taken = l.take(bytes);
    at.write(l, "a memory list")?;
    Ok(taken)
}

pub fn rewind_list(list: crate::pointers::GamePtr) -> Result<(), BadInput> {
    let at = list;
    let mut l = at.read::<MemoryList>("a memory list")?;
    l.rewind();
    at.write(l, "a memory list")
}
